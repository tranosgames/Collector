# Collector Binary
This tool can collect different artifact on running system.
Like a kape but faster, more secure and open source.


## Help command
```bash
This tool was an artefact collector fast and secure. It can collect low level files.

Usage: collector_cli.exe [OPTIONS]

Options:
  -s, --source <SOURCE>
          The source of collecting artefact [default: C:]
  -d, --destination <DESTINATION>
          The destination of collecting artefact [default: .\out\]
  -r, --ressources <RESSOURCES>
          Ressources selection [default: All]
  -p, --path-ressources <PATH_RESSOURCES>
          The path of artefact ressource collection [default: .\ressources\]
      --zip <ZIP_NAME>
          Zip output directory
      --vss
          Collect from vss
      --log
          Print log output in terminal. (Little bit longer)
  -v, --verbose
          Verbose log
  -h, --help
          Print help
  -V, --version
          Print version
```

## Next project

At the end of this project, I developping a GUI to interact easier with the binary.

## Features

- [X] VSS (Collect from volume shadow copy)
- [ ] Emebed config file and ressources to execute in click and lauch binary. 
- [ ] Add ZIP password
- [ ] Export to VHDX
- [ ] Can export to 7zip