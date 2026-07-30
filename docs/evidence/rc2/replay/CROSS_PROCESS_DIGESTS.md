# RC2 replay determinism evidence

Base commit: eb1396b904e8c27e892d995e4005f32a479cbfbc
Binary:     target/release/emit_trace_digest

## Cross-process runs

single_clean_track    run 1: e1a53500b56fe5a0288949ca8e8c858198071b0b23692859621df4994211fff7
single_clean_track    run 2: e1a53500b56fe5a0288949ca8e8c858198071b0b23692859621df4994211fff7
single_clean_track    run 3: e1a53500b56fe5a0288949ca8e8c858198071b0b23692859621df4994211fff7
crossing_two_tracks   run 1: 3391122bea5ab6c0c77ff6b4c4b3e18025973701f54f14c47f25aa4ba96ef64c
crossing_two_tracks   run 2: 3391122bea5ab6c0c77ff6b4c4b3e18025973701f54f14c47f25aa4ba96ef64c
crossing_two_tracks   run 3: 3391122bea5ab6c0c77ff6b4c4b3e18025973701f54f14c47f25aa4ba96ef64c
