// I have complete control over the interface, so I opted to go the path of least resistance and make all arguments named and not positional.
// Clap (structopt) does not allow an argument to be both positional and named so I would need to duplicate those arguments.
// In a real project I would do that anyway, for the ergonomics, but not necessarily as a priority.

use anyhow::{Error, Result};
use std::ops::RangeInclusive;
use std::str::FromStr;

/// A simple command-line interface to the cats registry.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    pub cmd: Cmd,
    /// Whether the output should be in JSON format.
    #[structopt(long, short)]
    pub json: bool,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Adds a new cat, producing the ID of the new cat.
    ///
    /// The name and age are required, but the breed is optional.
    Add {
        #[structopt(flatten)]
        cmd: CmdAdd,
    },
    /// Searches for an existing cat or set of cats.
    ///
    /// Each query parameter can be specified multiple times.
    /// With no parameters, this will simply return every cat.
    Find {
        #[structopt(flatten)]
        cmd: CmdFind,
    },
    /// Gets a cat or set of cats by ID.
    Get {
        /// The ID of the cat. May be specified multiple times.
        #[structopt(long, short, use_delimiter = true)]
        id: Vec<u64>,
    },
    /// Update a cat's information.
    Update {
        #[structopt(flatten)]
        cmd: CmdUpdate,
    },
    /// Removes a cat from the registry.
    Delete {
        /// The ID of the cat to remove.
        #[structopt(long, short)]
        id: u64,
    },
}

#[derive(Debug, StructOpt)]
pub struct CmdUpdate {
    /// The ID of the cat to update.
    #[structopt(long, short)]
    pub id: u64,
    /// The cat's new name.
    #[structopt(long, short)]
    pub name: Option<String>,
    /// The cat's new age.
    #[structopt(long, short)]
    pub age: Option<u32>,
    /// The cat's new breed.
    ///
    /// Make sure it's spelled correctly, because cat breeds change too often for the registry to
    /// know if it is a real breed.
    #[structopt(long, short)]
    pub breed: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct CmdFind {
    /// The name of the cat.
    #[structopt(long, short, use_delimiter = true)]
    pub name: Option<Vec<String>>,
    /// The age of the cat, in years.
    ///
    /// You can specify a range, e.g. 5-12
    #[structopt(long, short, use_delimiter = true)]
    pub age: Option<Vec<Age>>,
    /// The breed of the cat.
    #[structopt(long, short, use_delimiter = true)]
    pub breed: Option<Vec<String>>,
    /// Whether to search for cats that don't have a set breed.
    #[structopt(long, conflicts_with = "breed")]
    pub no_breed: bool,
    /// Whether to match the name and breed via fuzzy match.
    ///
    /// By default, they will be searched case insensitively but otherwise exact.
    #[structopt(long, short)]
    pub fuzzy: bool,
}

#[derive(Debug, StructOpt)]
pub struct CmdAdd {
    /// The name of the cat.
    #[structopt(long, short)]
    pub name: String,
    /// The age of the cat, in years.
    #[structopt(long, short)]
    pub age: u32,
    /// The breed of the cat.
    ///
    /// Make sure it's spelled correctly, because cat breeds change too often for the registry to
    /// know if it is a real breed.
    #[structopt(long, short)]
    pub breed: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Age {
    Range(RangeInclusive<u32>),
    Concrete(u32),
}

impl FromStr for Age {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some(divider) = s.find("-") {
            let lower = s[..divider].parse::<u32>()?;
            let upper = s[divider + 1..].parse::<u32>()?;
            Ok(Self::Range(lower..=upper))
        } else {
            let age = s.parse::<u32>()?;
            Ok(Self::Concrete(age))
        }
    }
}
