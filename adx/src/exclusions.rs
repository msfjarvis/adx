use crate::parse::get_packages;
use clap::{ValueEnum, builder::PossibleValue};
use std::fmt::Write;

#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub(crate) enum PrintType {
    IncludeGroup,
    IncludeModule,
}

impl ValueEnum for PrintType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::IncludeGroup, Self::IncludeModule]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            PrintType::IncludeGroup => Some(PossibleValue::new("group")),
            PrintType::IncludeModule => Some(PossibleValue::new("module")),
        }
    }
}

pub(crate) async fn print_inclusions(print_type: PrintType) {
    let packages = get_packages().await;
    let Ok(packages) = packages else { return };
    let mut rules = String::new();
    match print_type {
        PrintType::IncludeGroup => {
            packages
                .into_iter()
                .map(|pkg| pkg.group_id)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .for_each(|group_id| {
                    let _ = writeln!(rules, "includeGroup(\"{group_id}\")");
                });
        }
        PrintType::IncludeModule => {
            for pkg in packages {
                let _ = writeln!(
                    rules,
                    "includeModule(\"{}\", \"{}\")",
                    pkg.group_id, pkg.artifact_id
                );
            }
        }
    }
    println!(
        "
    dependencyResolutionManagement {{
      repositories {{
        exclusiveContent {{
          forRepository {{ google() }}
          filter {{
            {rules}
          }}
        }}
      }}
    }}
    ",
    );
}
