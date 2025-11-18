use cpu_info::CpuInfo;

fn main() {
    let cores = CpuInfo::new();


    println!("{:#?}", cores);
}