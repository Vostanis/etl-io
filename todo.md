# to do

- [ ] tests;
    - [ ] databases; ping, insert & delete from dbs
    - [ ] macros; check `macro == static`
- [ ] util;
    - [ ] generalised postgres query macro
    - [ ] postgres & scylladb versions of: `insert_doc()`
    - [ ] auto-serde function
    - [ ] pipe! macro `pipe!(I, O) == pipe_io::Pipe::<I, O>::new()`
- [ ] cli;
    - [ ] traditional; `pipe -i file.rs:input -o file.rs:output --conn user:password@localhost:8080`
    - [ ] menu; [inquire][1]
    - [ ] table preview; [comfy-table][2]
- [ ] benchmarks;
    - [ ] speed, using [hyperfine][3]
    - [ ] CPU power
    - [ ] comparison of `Deserialize vs. DeserializeOwned`

[1]: <https://github.com/mikaelmello/inquire> "visual applications for cli"
[2]: <https://github.com/Nukesor/comfy-table> "terminal tables"
[3]: <https://github.com/sharkdp/hyperfine> "general-purpose benchmarking tool"