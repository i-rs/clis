// i-rs-claw Dashboard — PM2 ecosystem file
//
// Usage:
//   1. Build with dashboard feature:
//      cargo build --features dashboard -p i-rs-claw --release
//
//   2. Start:
//      pm2 start crates/claw/ecosystem.config.js
//
//   3. View logs:
//      pm2 logs i-rs-claw-dashboard
//
// Config: edit ~/.i-rs-claw/config.toml  (port defaults to 3000)
//
// For development, replace `script` with:
//   script: 'cargo',
//   args: 'run --features dashboard -p i-rs-claw -- dashboard',
//   interpreter: 'none',

module.exports = {
  apps: [{
    name: 'i-rs-claw-dashboard',
    script: '/root/code/clis/target/release/i-rs-claw',
    args: 'dashboard',
    cwd: '/root/code/clis',
    exec_interpreter: 'none',
    exec_mode: 'fork',
    env: {
      RUST_LOG: 'info',
    },
    max_memory_restart: '500M',
    log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
    error_file: '/root/code/clis/logs/claw-error.log',
    out_file: '/root/code/clis/logs/claw-out.log',
    merge_logs: true,
    kill_timeout: 10000,
  }],
}
