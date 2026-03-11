#![allow(non_snake_case)]

use apdl_parser::{Dlist, Elist, Nlist, Prnsol, get_list};
use clap::Parser;
use math::{solve, sparse_sol};
use std::time::Instant;
use tracing::{debug, info};
use tracing_subscriber::filter;
use visualize::save_result_img;
mod visualize;
use cli::Cli;
mod cli;
mod math;


fn main() -> anyhow::Result<()> {
    let app_time = Instant::now();

    let cli = Cli::parse();
    let filter = match cli.log_lvl {
        cli::LogLvl::Info => filter::LevelFilter::INFO,
        cli::LogLvl::Debug => filter::LevelFilter::DEBUG,
        cli::LogLvl::Warn =>  filter::LevelFilter::WARN,
    };

    tracing_subscriber::fmt()
        .with_max_level(filter)
        .with_level(true)
        .init();

    info!("Parsing NLIST.lis");
    let nodes = get_list::<Nlist>(&cli.nlist)?;
    info!("Parsing ELIST.lis");
    let elems = get_list::<Elist>(&cli.elist)?;
    info!("Parsing DLIST.lis");
    let loads = get_list::<Dlist>(&cli.dlist)?;
    info!("Parsing PRNSOL.lis");
    let ansys = get_list::<Prnsol>(&cli.prnsol)?;

    info!("Nodes amount: {:?}", nodes.len());
    info!("Elems amount: {:?}", elems.len());

    debug!("\nNodes:\n{elems:?}");
    debug!("\nTriangles:\n{elems:?}");
    debug!("\nLoads:\n{loads:?}");

    let solve_time = Instant::now();

    let result = match cli.not_sparse {
        true => solve(&elems, &nodes, &loads, &cli).iter().map(|v| *v as f64).collect::<Vec<f64>>(),
        false => sparse_sol(&elems, &nodes, &loads, &cli),
    };
    info!("Gen matrix and solve time: {:.2?}", solve_time.elapsed());

    debug!("\nResult:\n{result:?}");

    save_result_img(&result, &elems, &nodes, &ansys, &cli);

    info!("App time: {:.2?}", app_time.elapsed());

    Ok(())
}
