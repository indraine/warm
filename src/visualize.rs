use plotly::{Configuration, ImageFormat, Layout, Plot, Scatter};
use colorgrad::{Color, Gradient, LinearGradient};
use apdl_parser::{Elist, Nlist, Prnsol};
use plotly::common::{Fill, Line, Mode};
use plotly::layout::{Axis, Legend, Annotation};
use plotly::common::Anchor;
use plotly::color::Rgb;
use chrono::prelude::*;
use tracing::info;
use crate::cli::Cli;

pub fn interpolate(x: f32, a: f32, b: f32) -> f32 {
    (x - a) / (b - a)
}

fn custom_result(
    result: &[f64],
    triangles: &[Elist],
    nodes: &[Nlist],
    gradient: &LinearGradient,
    plot: &mut Plot,
) {
    let T_MAX = max(result).unwrap();
    let T_MIN = min(result).unwrap();

    for elem in triangles {
        let node_i = &nodes[elem.node_i - 1];
        let node_j = &nodes[elem.node_j - 1];
        let node_k = &nodes[elem.node_k - 1];

        let T1 = result[node_i.node - 1];
        let T2 = result[node_j.node - 1];
        let T3 = result[node_k.node - 1];

        let temperature = (T1 + T2 + T3) / 3.0;

        let trace = Scatter::new(
            vec![node_i.x, node_j.x, node_k.x],
            vec![node_i.y, node_j.y, node_k.y],
        )
        .name("МКЭ Румянцева")
        .x_axis("x2")
        .y_axis("y2")
        .mode(Mode::None)
        .name(temperature.to_string())
        .show_legend(false)
        .fill_color(
            gradient
                .at(interpolate(temperature as f32, *T_MIN as f32, *T_MAX as f32))
                .to_hex_string(),
        )
        .fill(Fill::ToSelf);

        plot.add_trace(trace);
    }
}

fn max(nets: &[f64]) -> Option<&f64> {
    nets.iter().max_by(|a, b| a.total_cmp(b))
}

fn min(nets: &[f64]) -> Option<&f64> {
    nets.iter().min_by(|a, b| a.total_cmp(b))
}

fn ansys_result(
    ansys: &[Prnsol],
    triangles: &[Elist],
    nodes: &[Nlist],
    gradient: &LinearGradient,
    plot: &mut Plot,
) {
    let ansys_res = ansys.iter().map(|e| e.temp as f64).collect::<Vec<f64>>();

    let T_MAX = max(&ansys_res).unwrap();
    let T_MIN = min(&ansys_res).unwrap();

    for elem in triangles {
        let node_i = &nodes[elem.node_i - 1];
        let node_j = &nodes[elem.node_j - 1];
        let node_k = &nodes[elem.node_k - 1];

        let T1 = ansys_res[node_i.node - 1];
        let T2 = ansys_res[node_j.node - 1];
        let T3 = ansys_res[node_k.node - 1];

        let temperature = (T1 + T2 + T3) / 3.0;

        let trace = Scatter::new(
            vec![node_i.x, node_j.x, node_k.x],
            vec![node_i.y, node_j.y, node_k.y],
        )
        .name("Ansys APDL")
        .mode(Mode::None)
        .x_axis("x1")
        .y_axis("y1")
        .name(temperature.to_string())
        .show_legend(false)
        .fill_color(
            gradient
                .at(interpolate(temperature as f32, *T_MIN as f32, *T_MAX as f32))
                .to_hex_string(),
        )
        .fill(Fill::ToSelf);

        plot.add_trace(trace);
    }
}

fn difference_field(
    ansys: &[Prnsol],
    result: &[f64],
    triangles: &[Elist],
    nodes: &[Nlist],
    gradient: &LinearGradient,
    plot: &mut Plot,
) {
    let diff = ansys
        .iter()
        .zip(result)
        .map(|(a, b)| a.temp as f64 - *b)
        .collect::<Vec<f64>>();

    let T_MAX = max(&diff).unwrap();
    let T_MIN = min(&diff).unwrap();

    for elem in triangles {
        let node_i = &nodes[elem.node_i - 1];
        let node_j = &nodes[elem.node_j - 1];
        let node_k = &nodes[elem.node_k - 1];

        let T1 = diff[node_i.node - 1];
        let T2 = diff[node_j.node - 1];
        let T3 = diff[node_k.node - 1];

        let temperature = (T1 + T2 + T3) / 3.0;

        let trace = Scatter::new(
            vec![node_i.x, node_j.x, node_k.x],
            vec![node_i.y, node_j.y, node_k.y],
        )
        .name("Поле разницы")
        .mode(Mode::None)
        .x_axis("x3")
        .y_axis("y3")
        .name(temperature.to_string())
        .show_legend(false)
        .fill_color(
            gradient
                .at(interpolate(temperature as f32, *T_MIN as f32, *T_MAX as f32))
                .to_hex_string(),
        )
        .fill(Fill::ToSelf);

        plot.add_trace(trace);
    }
}

fn temps_result_diff(
    ansys: &[Prnsol],
    result: &[f64],
    plot: &mut Plot,
) {
    let ansys_temps = ansys.iter().map(|v| v.temp).collect::<Vec<_>>();
    let result_temps = result.into_iter().copied().collect::<Vec<_>>();

    let xs = (0..result_temps.len()).collect::<Vec<_>>();
    let trace1 = Scatter::new(xs.clone(), ansys_temps)
        .name("Ansys")
        .mode(Mode::None)
        .x_axis("x4")
        .y_axis("y4")
        .show_legend(true)
        .mode(Mode::Lines)
        .line(Line::new().color(Rgb::new(255, 0, 0)))
        .fill_color("#aaffaa");

    let trace2 = Scatter::new(xs, result_temps)
        .name("МКЭ Румянцева")
        .mode(Mode::None)
        .x_axis("x4")
        .y_axis("y4")
        .line(Line::new().color(Rgb::new(0, 0, 255)))
        .show_legend(true)
        .mode(Mode::Lines);

    plot.add_trace(trace1);
    plot.add_trace(trace2);
}

pub fn save_result_img(
    result: &[f64],
    triangles: &[Elist],
    nodes: &[Nlist],
    ansys: &[Prnsol],
    cli: &Cli,
) {

    info!("Generate html");
    let gradient = colorgrad::GradientBuilder::new()
        .colors(&[
            Color::from_rgba8(0, 0, 255, 255),
            Color::from_rgba8(17, 159, 242, 255),
            Color::from_rgba8(7, 238, 252, 255),
            Color::from_rgba8(1, 232, 218, 255),
            Color::from_rgba8(7, 235, 9, 255),
            Color::from_rgba8(149, 240, 19, 255),
            Color::from_rgba8(235, 237, 3, 255),
            Color::from_rgba8(241, 194, 21, 255),
            Color::from_rgba8(255, 0, 0, 255),
        ])
        .build::<colorgrad::LinearGradient>()
        .unwrap();

    let mut plot = Plot::new();

    plot.set_configuration(Configuration::new());

    ansys_result(ansys, triangles, nodes, &gradient, &mut plot);
    custom_result(result, triangles, nodes, &gradient, &mut plot);
    difference_field(ansys, result, triangles, nodes, &gradient, &mut plot);
    temps_result_diff(ansys, result, &mut plot);

    let dt = Local::now().format("%A %e %B %Y, %T");

    let layout = Layout::new()
        .title(format!(
            "Теплопроводность. Ansys APDL vs МКЭ Румянцева \n{:?}",
            dt.to_string()
        ))
        .legend(
            Legend::new()
                .y_anchor(plotly::common::Anchor::Top)
                .x_anchor(plotly::common::Anchor::Right)
                .y(0.43)
                .x(0.98)
                .border_width(1),
        )
        .height(786)
        .width(1024)
        // верхний ряд полей: Ansys, МКЭ, разность
        .x_axis(Axis::new().domain(&[0.02, 0.30]).anchor("y1"))
        .y_axis(Axis::new().domain(&[0.55, 1.]).anchor("x1"))
        .x_axis2(Axis::new().domain(&[0.36, 0.64]).anchor("y2"))
        .y_axis2(Axis::new().domain(&[0.55, 1.]).anchor("x2"))
        .x_axis3(Axis::new().domain(&[0.70, 0.98]).anchor("y3"))
        .y_axis3(Axis::new().domain(&[0.55, 1.]).anchor("x3"))
        .x_axis4(
            Axis::new()
                .domain(&[0., 1.])
                .anchor("y4")
                .title("Индекс узла"),
        )
        // нижний график: сравнение температур по узлам
        .y_axis4(
            Axis::new()
                .domain(&[0., 0.40])
                .anchor("x4")
                .title("Температура"),
        )
        .annotations(vec![
            Annotation::new()
                .text("Ansys APDL")
                .x(0.16)
                .y(1.03)
                .x_ref("paper")
                .y_ref("paper")
                .x_anchor(Anchor::Center)
                .show_arrow(false),
            Annotation::new()
                .text("МКЭ Румянцева")
                .x(0.50)
                .y(1.03)
                .x_ref("paper")
                .y_ref("paper")
                .x_anchor(Anchor::Center)
                .show_arrow(false),
            Annotation::new()
                .text("Поле разницы")
                .x(0.84)
                .y(1.03)
                .x_ref("paper")
                .y_ref("paper")
                .x_anchor(Anchor::Center)
                .show_arrow(false),
            Annotation::new()
                .text("Сравнение температур по узлам")
                .x(0.5)
                .y(0.49)
                .x_ref("paper")
                .y_ref("paper")
                .x_anchor(Anchor::Center)
                .show_arrow(false),
        ]);

    plot.set_layout(layout);

    match cli.image {
        crate::cli::ImageFormat::SVG => plot.show_image(ImageFormat::SVG, 1000, 750),
        crate::cli::ImageFormat::PNG => plot.show_image(ImageFormat::PNG, 1000, 750),
        crate::cli::ImageFormat::WEBP => plot.show_image(ImageFormat::WEBP, 1000, 750),
        crate::cli::ImageFormat::JPEG => plot.show_image(ImageFormat::JPEG, 1000, 750),
        crate::cli::ImageFormat::None => {}
    }

    if !cli.web_off {
        plot.show()
    }
}
