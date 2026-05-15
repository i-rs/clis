use clap::Args;
use i_rs_core::presentation::OutputFormat;

#[derive(Args)]
pub struct ExampleArgs {}

pub fn execute(_args: &ExampleArgs, _format: &OutputFormat) -> anyhow::Result<()> {
    println!(r"
i-rs-tax 使用示例:

1. 添加个人所得税记录:
   i-rs-tax add 个人所得税2024 \
     --tax-type personal \
     --amount 12000 \
     --date 2024-03-15 \
     --status filed \
     --tag 工资 \
     --tag 年终奖

2. 添加增值税记录:
   i-rs-tax add 增值税Q1 \
     --tax-type vat \
     --amount 5000 \
     --date 2024-04-01 \
     --status paid \
     --tag 季度申报

3. 查看所有税务记录:
   i-rs-tax list

4. 按年度查看记录:
   i-rs-tax list --year 2024

5. 按标签过滤:
   i-rs-tax list --tag 年终奖

6. 按税种过滤:
   i-rs-tax list --tax-type personal

7. 查看记录详情:
   i-rs-tax get 个人所得税2024

8. 删除记录:
   i-rs-tax delete 个人所得税2024

9. 查看年度统计:
   i-rs-tax stats
   i-rs-tax stats --year 2024

10. JSON 格式输出:
    i-rs-tax list --json
    i-rs-tax get 个人所得税2024 --json
    i-rs-tax stats --year 2024 --json
");
    Ok(())
}
