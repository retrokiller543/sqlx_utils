#![allow(unused_assignments, unused_mut)]

use rustc_version::{Channel, version_meta};

fn check_db_features() {
    let mut db_feature: Option<Vec<&str>> = None;

    #[cfg(feature = "any")]
    {
        db_feature = Some(vec!["any"]);
    }

    #[cfg(feature = "postgres")]
    {
        if let Some(ref mut enabled_by) = db_feature {
            enabled_by.push("postgres")
        } else {
            db_feature = Some(vec!["postgres"]);
        }
    }

    #[cfg(feature = "mysql")]
    {
        if let Some(ref mut enabled_by) = db_feature {
            enabled_by.push("mysql")
        } else {
            db_feature = Some(vec!["mysql"]);
        }
    }

    #[cfg(feature = "sqlite")]
    {
        if let Some(ref mut enabled_by) = db_feature {
            enabled_by.push("sqlite")
        } else {
            db_feature = Some(vec!["sqlite"]);
        }
    }

    if let Some(enabled_by) = db_feature {
        if enabled_by.len() > 1 {
            let features = enabled_by
                .iter()
                .map(|feature| format!("{:?}", feature))
                .collect::<Vec<String>>()
                .join(", ");

            println!(
                "cargo:warning=Multiple database features enabled: {}, potential conflicts can occur. Falling back to 'any' feature",
                features
            );
            println!("cargo:rustc-cfg=feature=\"any\"");
            println!("cargo:rustc-env=DATABASE_FEATURE=any");
        }
    } else {
        panic!(
            "No database feature enabled, please enable one of the following: `any`, `postgres`, `mysql`, `sqlite`"
        )
    }
}

fn main() {
    let channel = match version_meta().unwrap().channel {
        Channel::Stable => "CHANNEL_STABLE",
        Channel::Beta => "CHANNEL_BETA",
        Channel::Nightly => "CHANNEL_NIGHTLY",
        Channel::Dev => "CHANNEL_DEV",
    };
    println!("cargo:rustc-cfg={}", channel);

    check_db_features();
}
