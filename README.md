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
    current-as-csv    Lookup all known items in the provided database that have been seen in the last 30 days
    help              Prints this message or the help of the given subcommand(s)
    run               Check for new merchandise, storing the item data and timestamp for this check
```

### `run`

```sh
titan-fitness-stock-run 0.1.0
Check for new merchandise, storing the item data and timestamp for this check

USAGE:
    titan-fitness-stock run --db <db-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --db <db-path>    The path to a database file. On first run, this will establish your database so when it is run
                          again, it can compare the current available stock
```

### `current-as-csv`

```sh
titan-fitness-stock-current-as-csv 0.1.0
Lookup all known items in the provided database that have been seen in the last 30 days

USAGE:
    titan-fitness-stock current-as-csv [OPTIONS] --db <db-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --db <db-path>    The path to a previously created database file. Execute the `run` command to create one
    -o, --out <out>       A path to an output file, if not provided csv will be printed to stdout
```

__CSV Columns__

| last seen      | name   | id                             | price                          | category                             | brand                             | position                                         | list                    | link                                              | back_in_stock |
| -------------- | ------ | ------------------------------ | ------------------------------ | ------------------------------------ | --------------------------------- | ------------------------------------------------ | ----------------------- | ------------------------------------------------- | ------------- |
| unix timestamp | string | item or group's id as a string | price or range of prices (USD) | `/` seperated list of category names | Internal brand name, may be empty | list of metadata, sometimes includes cateogories | url for the item's page | true if it was last seen in stock otherwise false |

## Database

The database for this application is a single file embedded database provided by the [Structsy project](http://structsy.rs/)
