use clap::{Parser, ValueEnum, command};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
///
/// Программа для расчёта теплопроводности методом конечных
/// элементов без учёта конвекций по терории Румянцева и
/// данным файлов, сгенерированных `Ansys APDL`.
///  Файлы: `NLIST.lis`, `ELIST.lis`, `DLIST.lis`, `PRNSOL.lis`
///
pub struct Cli {
    #[arg(short, long, default_value = "NLIST.lis")]
    /// Путь до файла NLIST.lis
    pub nlist: PathBuf,

    #[arg(short, long, default_value = "ELIST.lis")]
    /// Путь до файла ELIST.lis
    pub elist: PathBuf,

    #[arg(short, long, default_value = "DLIST.lis")]
    /// Путь до файла DLIST.lis
    pub dlist: PathBuf,

    #[arg(short, long, default_value = "PRNSOL.lis")]
    /// Путь до файла PRNSOL.lis
    pub prnsol: PathBuf,

    #[arg(short, long, default_value_t = false)]
    /// Открыть визуализацию в браузере
    pub web_off: bool,

    #[arg(short('N'), long, default_value_t = false)]
    // Считать LU разложением для НЕразреженных матриц
    pub not_sparse: bool,

    #[arg(short('D'), long, default_value = "klu")]
    /// Вид матричного разложения
    pub decomposition: Decomposition,

    #[arg(long, default_value = "0.0001")]
    /// Первый коэфициент теплопроводноти
    pub lambda_xx: f32,

    #[arg(long, default_value = "0.0001")]
    /// Второй коэфициент теплопроводноти
    pub lambda_yy: f32,

    #[clap(short, long, default_value = "info")]
    /// Уровень логирования
    pub log_lvl: LogLvl,

    #[clap(short, long, default_value = "none")]
    /// Формат сохраняемого изображения
    pub image: ImageFormat,
}

//TODO: Почему нельзя использовать plotly::ImageFormat?
#[derive(ValueEnum, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum ImageFormat {
    None,
    SVG,
    PNG,
    WEBP,
    JPEG,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum LogLvl {
    Info,
    Debug,
    Warn,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum Decomposition {
    Klu,
    Umfpack,
    Mumps
}
