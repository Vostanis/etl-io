# to do

- [ ] tests;
    - [ ] databases; ping, insert & delete from dbs
    - [ ] macros; check `macro == static`
    - [ ] datasources; e.g., SEC company tickers, Binance, etc. (to be defined)
- [ ] util;
    - [ ] generalised postgres query macro
    - [ ] postgres & scylladb versions of: `insert_doc()`
    - [ ] auto-serde function
- [ ] cli;
    - [ ] traditional; `pipe -i file.rs:input -o file.rs:output --conn user:password@localhost:8080`
    - [ ] menu; `inquire` crate
    - [ ] table preview; `comfy-table` crate
- [ ] benchmarks;
    - [ ] speed, using [hyperfine][1]
    - [ ] CPU power
    - [ ] comparison of `Deserialize vs. DeserializeOwned`

[1]: <https://github.com/sharkdp/hyperfine> "<hyperfine> general-purpose brenchmarking tool"