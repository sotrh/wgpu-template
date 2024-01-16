use pollster::FutureExt;
use wgpu_template::run;

fn main() -> anyhow::Result<()> {
   run().block_on()
}