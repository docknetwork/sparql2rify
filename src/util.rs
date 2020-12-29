use crate::types::{InvalidRule, RdfNode};
use rify::{Claim, Entity};
use std::collections::BTreeSet;

pub fn as_blank(ent: &Entity<String, RdfNode>) -> Option<&str> {
    match ent {
        Entity::Bound(RdfNode::Blank(name)) => Some(&**name),
        _ => None,
    }
}

pub fn as_unbound(ent: &Entity<String, RdfNode>) -> Option<&str> {
    match ent {
        Entity::Unbound(name) => Some(&**name),
        _ => None,
    }
}

/// convert blank nodes to unbound variables, in order to prevent naming collisions
/// we first ensure no blank nodes have the same name as an unbound variable
pub fn unbind_blanks(
    if_all: &mut [Claim<Entity<String, RdfNode>>],
    then: &mut [Claim<Entity<String, RdfNode>>],
) -> Result<(), InvalidRule> {
    // check
    let ents = if_all.iter().chain(&*then).flatten();
    let blanks: BTreeSet<&str> = ents.clone().filter_map(as_blank).collect();
    let unbound: BTreeSet<&str> = ents.filter_map(as_unbound).collect();
    if let Some(name) = blanks.intersection(&unbound).next() {
        let name = name.to_string();
        return Err(InvalidRule::NameCollision { name });
    }

    // execute
    for ent in if_all.iter_mut().chain(then).flatten() {
        if let Some(name) = as_blank(&*ent) {
            *ent = Entity::Unbound(name.to_string());
        }
    }

    Ok(())
}
