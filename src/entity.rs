use super::*;

use std::{
    convert::TryInto,
    ops::BitOrAssign,
};

#[derive(Copy, Clone, Debug)]
pub struct Entity {
    idx: u32,
    gen: u32,
}

impl Entity {
    pub(crate) unsafe fn placeholder() -> Self {
        Self { idx: 0, gen: 0 }
    }

    pub(crate) fn idx(&self) -> usize { self.idx as usize }
    pub(crate) fn gen(&self) -> usize { self.gen as usize }
}

#[derive(Debug)]
pub(crate) struct Entry {
    gen: u32,
    filled: bool,
    pub(crate) comp_mask: BitMask,
}

#[derive(Default)]
pub struct Entities {
    entities: Vec<Entry>,
}

impl Entities {
    pub fn create(&mut self) -> Entity {
        let (idx, gen) = if let Some((idx, entry)) = self.entities
            .iter_mut()
            .enumerate()
            .find(|(_, entry)| entry.gen < u32::MAX && !entry.filled)
        {
            entry.gen += 1;
            entry.filled = true;
            entry.comp_mask = BitMask::zero();
            (idx, entry.gen)
        } else {
            let idx = self.entities.len();
            self.entities.push(Entry {
                gen: 0,
                filled: true,
                comp_mask: BitMask::zero(),
            });
            (idx, 0)
        };

        Entity {
            idx: idx
                .try_into()
                .unwrap_or_else(|_| panic!("No more entity slots may be allocated!")),
            gen,
        }
    }

    pub fn delete(&mut self, entity: Entity) {
        if let Some(entry) = self.entities.get_mut(entity.idx()) {
            if entry.filled && entry.gen == entity.gen {
                entry.filled = false;
            }
        } else {
            unreachable!("Invariant violated: entity index must always be valid");
        }
    }

    pub(crate) fn entry(&self, entity: Entity) -> Option<&Entry> {
        self.entities
            .get(entity.idx())
            .filter(|entry| entry.gen == entity.gen)
    }

    pub(crate) fn entry_mut(&mut self, entity: Entity) -> Option<&mut Entry> {
        self.entities
            .get_mut(entity.idx())
            .filter(|entry| entry.gen == entity.gen)
    }

    pub(crate) fn comp_mask(&self, entity: Entity) -> Option<&BitMask> {
        self.entry(entity).map(|entry| &entry.comp_mask)
    }

    pub(crate) fn iter_filter<'a>(&'a self, filter: &'a (BitMask, BitMask)) -> EntityIter<'a> {
        // println!("Entities = {}", self.entities.len());
        // println!("Mask = {:?}", filter);
        self.entities
            .iter()
            .enumerate()
            .filter(move |(i, entry)| {
                // println!("Entry[{}] = {:?}", i, entry);
                entry.filled && entry.comp_mask.matches(filter)
            })
            .map(|(idx, entry)| Entity {
                idx: idx as u32,
                gen: entry.gen,
            })
    }
}

pub type EntityIter<'a> = impl Iterator<Item = Entity> + 'a;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitMask(u64);

impl BitMask {
    pub fn zero() -> Self { Self(0) }

    pub fn with(x: u64) -> Self { Self(1 << x) }

    pub fn intersection(self, other: Self) -> Self { Self(self.0 & other.0) }

    pub fn union(self, other: Self) -> Self { Self(self.0 | other.0) }

    pub fn matches(&self, (check, mask): &(Self, Self)) -> bool {
        self.0 & mask.0 == check.0
    }

    pub fn bit_is_set(&self, x: u64) -> bool {
        (self.0 >> x) & 1 != 0
    }

    pub fn set_bit(&mut self, x: u64) {
        self.0 |= 1 << x;
    }

    pub fn unset_bit(&mut self, x: u64) {
        self.0 &= !(1 << x);
    }

    pub fn combine_filters(
        (a_check, a_mask): (BitMask, BitMask),
        (b_check, b_mask): (BitMask, BitMask),
    ) -> Option<(BitMask, BitMask)>
    {
        let check = a_check.clone().union(b_check.clone());
        if check.clone().intersection(a_mask.clone()) == a_check
            && check.clone().intersection(b_mask.clone()) == b_check
        {
            Some((check, a_mask.union(b_mask)))
        } else {
            None
        }
    }
}

impl BitOrAssign for BitMask {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}
