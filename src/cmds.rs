// The code here is carefully written not to leak user input into concatenated SQL.
// However, a future maintainer might not be so careful.
// To that end, if this were a real project I would have put effort into
// clearly demarcating where expressions go into SQL and where they go into prepared parameters.
// It would also have inline comments throughout.

// The module separation is good enough to have a place to put code without having a god-file.
// However, in a real project I would further separate the modules, so that cmds does not interact with args.

use crate::args::{Age, CmdAdd, CmdFind, CmdUpdate};
use crate::Printable;
use anyhow::Result;
use itertools::Itertools;
use prettytable::Table;
use rusqlite::{Connection, Row, ToSql};
use std::borrow::Cow;
use std::fmt::Display;
use std::io;
use std::iter;

pub fn add(conn: &Connection, cmd: CmdAdd) -> Result<Cat> {
    Ok(conn.query_row(
        "INSERT INTO cats (name, age, breed) VALUES (?, ?, ?) RETURNING *",
        params![cmd.name, cmd.age, cmd.breed],
        Cat::from_row,
    )?)
}

impl Printable for Option<Cat> {
    fn print_display(&self) {
        if let Some(cat) = self {
            cat.print_display()
        } else {
            println!("No such cat exists")
        }
    }
    fn print_plain(&self) {
        if let Some(cat) = self {
            cat.print_plain()
        }
    }
    fn print_json(&self) {
        if let Some(cat) = self {
            cat.print_json()
        } else {
            println!("{{}}")
        }
    }
}

pub fn delete(conn: &Connection, id: u64) -> Result<Option<Cat>> {
    let mut stmt = conn.prepare("DELETE FROM cats WHERE id = ? RETURNING *")?;
    let mut rows = stmt.query_map([id], Cat::from_row)?;
    Ok(rows.next().transpose()?)
}

pub fn get(conn: &Connection, id: &[u64]) -> Result<Vec<Cat>> {
    let mut stmt = String::from("SELECT * FROM cats WHERE ");
    stmt.push_str(&id.iter().map(|_| "id = ?").join(" OR "));
    conn.prepare(&stmt)?
        .query_map(rusqlite::params_from_iter(id), Cat::from_row)?
        .map(|res| Ok(res?))
        .collect()
}

pub fn find(conn: &Connection, cmd: CmdFind) -> Result<Vec<Cat>> {
    let mut params_owned = Vec::new();
    let mut params = Vec::new();
    let fuzzy = cmd.fuzzy;
    let name_clause = cmd.name.map(|names| {
        let len = names.len();
        if fuzzy {
            params_owned.extend(names);
            format!(
                "name IN VALUES ({})",
                iter::repeat("?").take(len).join(", ")
            )
        } else {
            params_owned.extend(names.into_iter().map(|name| format!("%{}%", name)));
            format!("({})", iter::repeat("name LIKE ?").take(len).join(" OR "))
        }
    });
    let breed_clause = cmd.breed.map(|breeds| {
        let len = breeds.len();
        if fuzzy {
            params_owned.extend(breeds);
            format!(
                "breed IN VALUES ({})",
                iter::repeat("?").take(len).join(", ")
            )
        } else {
            params_owned.extend(breeds.into_iter().map(|breed| format!("%{}%", breed)));
            format!("({})", iter::repeat("breed LIKE ?").take(len).join(" OR "))
        }
    });
    params.extend(params_owned.iter().map(|x| x as &dyn ToSql));
    let age_clause = cmd.age.as_ref().map(|ages| {
        format!(
            "({})",
            ages.iter()
                .map(|age| {
                    match age {
                        Age::Concrete(age) => {
                            params.push(age);
                            "age = ?"
                        }
                        Age::Range(range) => {
                            params.push(range.start());
                            params.push(range.end());
                            "age BETWEEN ? AND ?"
                        }
                    }
                })
                .join(" OR ")
        )
    });
    let no_breed_clause = cmd.no_breed.then(|| "breed ISNULL");
    let clauses = [name_clause.as_deref(), age_clause.as_deref(), breed_clause.as_deref(), no_breed_clause]
        .iter()
        .flatten()
        .join(" AND ");
    let stmt = if clauses.is_empty() {
        Cow::Borrowed("SELECT * FROM cats")
    } else {
        Cow::Owned(format!("SELECT * FROM cats WHERE {}", clauses))
    };
    conn.prepare(&stmt)?
        .query_map(&*params, Cat::from_row)?
        .map(|res| Ok(res?))
        .collect()
}

pub fn update(conn: &Connection, cmd: CmdUpdate) -> Result<Option<Cat>> {
    let mut stmt = String::from("UPDATE cats SET ");
    let mut params = Vec::new();
    let name_clause = cmd.name.as_ref().map(|name| {
        params.push(name as &dyn ToSql);
        "name = ?"
    });
    let age_clause = cmd.age.as_ref().map(|age| {
        params.push(age);
        "age = ?"
    });
    let breed_clause = cmd.breed.as_ref().map(|breed| {
        params.push(breed);
        "breed = ?"
    });
    stmt.push_str(
        &[name_clause, age_clause, breed_clause]
            .iter()
            .flatten()
            .join(", "),
    );
    stmt.push_str(" WHERE id = ? RETURNING *");
    params.push(&cmd.id);
    let mut stmt = conn.prepare(&stmt)?;
    let mut rows = stmt.query_map(&*params, Cat::from_row)?;
    Ok(rows.next().transpose()?)
}

#[derive(Debug, Serialize)]
pub struct Cat {
    pub id: u64,
    pub name: String,
    pub age: u32,
    pub breed: Option<String>,
}

impl Cat {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            age: row.get(2)?,
            breed: row.get(3)?,
        })
    }
}

impl Printable for Cat {
    fn print_display(&self) {
        let mut table = table!([
            self.id,
            self.name,
            self.age,
            self.breed.as_deref().unwrap_or("<none>")
        ]);
        table.set_titles(["ID", "Name", "Age", "Breed"].iter().collect());
        table.printstd();
    }
    fn print_plain(&self) {
        println!(
            "{} {} {} {}",
            self.id,
            self.name,
            self.age,
            self.breed.as_deref().unwrap_or("<none>")
        )
    }
    fn print_json(&self) {
        serde_json::to_writer(io::stdout(), self).unwrap();
    }
}

impl Printable for Vec<Cat> {
    fn print_display(&self) {
        if self.is_empty() {
            None::<Cat>.print_display();
            return;
        }
        let mut table = Table::new();
        table.set_titles(["ID", "Name", "Age", "Breed"].iter().collect());
        for cat in self {
            table.add_row(
                [
                    &cat.id as &dyn Display,
                    &cat.name,
                    &cat.age,
                    &cat.breed.as_deref().unwrap_or("<none>"),
                ]
                .iter()
                .collect(),
            );
        }
        table.printstd();
    }
    fn print_plain(&self) {
        for cat in self {
            cat.print_plain()
        }
    }
    fn print_json(&self) {
        serde_json::to_writer(io::stdout(), self).unwrap();
    }
}
