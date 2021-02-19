# Titan Fitness Stock

A Cli application for tracking what is in stock at [Titan Fitness](https://titan.fitness)



## Usage

```sh
titan-fitness-stock 0.1.0

USAGE:
    titan-fitness-stock <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    current-as-csv    
    help              Prints this message or the help of the given subcommand(s)
    run
```

### `run`

```sh
titan-fitness-stock-run 0.1.0

USAGE:
    titan-fitness-stock run --db <db-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --db <db-path> 
```

### `current-as-csv`

```sh
titan-fitness-stock-current-as-csv 0.1.0

USAGE:
    titan-fitness-stock current-as-csv [OPTIONS] --db <db-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --db <db-path>    
    -o, --out <out> 
```
