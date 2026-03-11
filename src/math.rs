use nalgebra::{Const, DMatrix, DVector, Dyn, LU, Matrix, Matrix3, VecStorage};
use apdl_parser::{Dlist, Elist, Nlist};
use crate::cli::{Cli, Decomposition};
use russell_sparse::prelude::*;
use tracing::{debug, info};
use russell_lab::{Vector};

pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn triangle_square(a: Point, b: Point, c: Point) -> f32 {
    0.5 * ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs()
}

/// Симметричная числовая матрица
/// 
/// * `a` - первая точка элемента
/// * `b` - вторая точка элемента
/// * `c` - третья точка элемента
/// 
pub fn c_e(a: &Nlist, b: &Nlist, c: &Nlist) -> Matrix3<f32> {
    let x_i = a.x;
    let x_j = b.x;
    let x_k = c.x;

    let c_i = x_k - x_j;
    let c_j = x_i - x_k;
    let c_k = x_j - x_i;

    Matrix3::new(
        c_i.powi(2),
        c_i * c_j,
        c_i * c_k,
        c_i * c_j,
        c_j.powi(2),
        c_j * c_k,
        c_i * c_k,
        c_j * c_k,
        c_k.powi(2),
    )
}


/// Симметричная числовая матрица
/// 
/// * `a` - первая точка элемента
/// * `b` - вторая точка элемента
/// * `c` - третья точка элемента
/// 
pub fn b_e(a: &Nlist, b: &Nlist, c: &Nlist) -> Matrix3<f32> {
    let y_i = a.y;
    let y_j = b.y;
    let y_k = c.y;

    let b_i = y_j - y_k;
    let b_j = y_k - y_i;
    let b_k = y_i - y_j;

    Matrix3::new(
        b_i.powi(2),
        b_i * b_j,
        b_i * b_k,
        b_i * b_j,
        b_j.powi(2),
        b_j * b_k,
        b_i * b_k,
        b_j * b_k,
        b_k.powi(2),
    )
}

///
/// Матрица жесткости для элемента
///
/// * `b_e` - симметричная числовая матрица: 
///                 - слайд №7   docs/МКЭ_в_двумерных_задачах_теплопроводности_и_теории_упругости.pdf
///                 - стр. 65-66 docs/МКЭ_в_задачах_теплопроводности_Румянцев.pdf
/// 
/// * `c_e` - тоже что и `b_e`
/// * `delta` - площадь элемента (треугольника)
/// * `lambda_xx` - первый коэффициент теплопроводности
/// * `lambda_yy` - второй коэффициент теплопроводности
///
pub fn element_stiffness_matrix(
    b_e: Matrix3<f32>,
    c_e: Matrix3<f32>,
    delta: f32,
    A: f32,
    lambda_xx: f32,
    lambda_yy: f32,
) -> Matrix3<f32> {
    (delta / (4.0 * A)) * (lambda_xx * b_e + lambda_yy * c_e)
}

pub fn get_stiffness_matrix(elem: &Elist, nodes: &[Nlist], lambda_xx: f32, lambda_yy: f32) -> Matrix3<f32> {
    let node_i = &nodes[elem.node_i - 1];
    let node_j = &nodes[elem.node_j - 1];
    let node_k = &nodes[elem.node_k - 1];

    let b_e = b_e(node_i, node_j, node_k);
    let c_e = c_e(node_i, node_j, node_k);

    let a: Point = Point {
        x: node_i.x,
        y: node_i.y,
    };

    let b: Point = Point {
        x: node_j.x,
        y: node_j.y,
    };

    let c: Point = Point {
        x: node_k.x,
        y: node_k.y,
    };

    let square = triangle_square(a, b, c);

    element_stiffness_matrix(b_e, c_e, square, 1.0, lambda_xx, lambda_yy)
}

pub fn get_global_stiffness_matrix(triangles: &[Elist], nodes: &[Nlist], lambda_xx: f32, lambda_yy: f32) -> DMatrix<f32> {
    info!("Generating global stiffness matrix");

    let n = nodes.len();
    let mut A = DMatrix::<f32>::zeros(n, n);

    for elem in triangles {
        let matrix = get_stiffness_matrix(elem, nodes, lambda_xx, lambda_yy);

        let indicies_set = [elem.node_i, elem.node_j, elem.node_k];
        let len = indicies_set.len();

        for i in 0..len {
            for j in 0..len {
                A[(indicies_set[i] - 1, indicies_set[j] - 1)] += matrix[(i, j)];
            }
        }
    }

    A
}

pub fn attach_loads(
    mut A: DMatrix<f32>,
    nodes: &[Nlist],
    loads: &[Dlist],
) -> (DMatrix<f32>, DVector<f32>) {
    let len = nodes.len();
    let mut B: DVector<f32> = DVector::zeros(len);

    for load in loads {
        let row = 0.0 * A.row(load.node - 1);

        A.set_row(load.node - 1, &row);

        A[(load.node - 1, load.node - 1)] = 1.0;
        B[load.node - 1] = load.real;
    }

    (A, B)
}

pub fn solve(
    triangles: &[Elist],
    nodes: &[Nlist],
    loads: &[Dlist],
    cli: &Cli,
) -> Matrix<f32, Dyn, Const<1>, VecStorage<f32, Dyn, Const<1>>> {
    let A = get_global_stiffness_matrix(triangles, nodes, cli.lambda_xx, cli.lambda_yy);
    let (A, B) = attach_loads(A, nodes, loads);

    debug!("General stiffness matrix:\n{A}");
    debug!("Loads vector\n{B}");

    info!("Solving linear system at LU decomposition");

    LU::new(A).solve(&B).unwrap()
}

pub fn sparse_sol(
    triangles: &[Elist],
    nodes: &[Nlist],
    loads: &[Dlist],
    cli: &Cli,
) -> Vec<f64> {

    let A = get_global_stiffness_matrix(triangles, nodes, cli.lambda_xx, cli.lambda_yy);
    let (A, B) = attach_loads(A, nodes, loads);

    let mut nnz = 0;
    for elem in A.iter().copied() {
        if elem != 0.0 {
            nnz += 1;
        }
    }

    let (nrow, ncol) = (A.nrows(), A.ncols());

    let mut mat = CooMatrix::new(nrow, ncol, nnz, Sym::No).unwrap();
    for i in 0..A.nrows() {
        for j in 0..A.ncols() {
            let val = A[(i, j)];
            if val != 0.0 {
                let _ = mat.put(i, j, val as f64);
            }
        }
    }

    let mut x = Vector::new(nrow);

    let _ = match cli.decomposition {
        Decomposition::Klu => LinSolver::compute(Genie::Klu, &mut x, &mat, &Vector::from(&B.as_slice()), None).unwrap(),
        Decomposition::Umfpack => LinSolver::compute(Genie::Umfpack, &mut x, &mat, &Vector::from(&B.as_slice()), None).unwrap(),
        Decomposition::Mumps => LinSolver::compute(Genie::Mumps, &mut x, &mat, &Vector::from(&B.as_slice()), None).unwrap(),
    };

    x.into_iter().collect::<Vec<_>>()
}