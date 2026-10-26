```bash
cargo clippy --bin ragasukma -- \
  -W dead_code \
  -W unused \
  -W clippy::redundant_clone \
  -W clippy::await_holding_lock \
  -W clippy::mutex_atomic \
  -W clippy::rc_buffer \
  -W clippy::panic \
  -W clippy::unwrap_used
  
npx tsc --noEmit
npx eslint .
npx knip
npm run build
```
