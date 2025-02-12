use serde::{Deserialize, Serialize};

use crate::crust::{
    iow::{BurnWitness, MintWitness},
    TxWitness,
};
use crate::{Digest, Hash};

pub type Value = u64;
pub type Unit = [u8; 32];
pub const NOP_VK: [u8; 32] = [0u8; 32];

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UnitWitness {
    pub spending_covenant: [u8; 32],
    pub minting_covenant: [u8; 32],
    pub burning_covenant: [u8; 32],
}

impl UnitWitness {
    pub fn unit(&self) -> Unit {
        let mut hasher = Hash::new();
        hasher.update(b"NOMOS_CL_UNIT");
        hasher.update(&self.spending_covenant);
        hasher.update(&self.minting_covenant);
        hasher.update(&self.burning_covenant);

        hasher.finalize().into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct UnitBalance {
    pub unit: Unit,
    pub pos: u64,
    pub neg: u64,
}

impl UnitBalance {
    pub fn is_zero(&self) -> bool {
        self.pos == self.neg
    }

    pub fn pos(unit: Unit, value: u64) -> Self {
        Self {
            unit,
            pos: value,
            neg: 0,
        }
    }

    pub fn neg(unit: Unit, value: u64) -> Self {
        Self {
            unit,
            pos: 0,
            neg: value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct Balance {
    pub balances: Vec<UnitBalance>,
}
impl Balance {
    pub fn zero() -> Self {
        Self {
            balances: Vec::new(),
        }
    }

    pub fn from_tx(tx: &TxWitness) -> Self {
        let mut balance = Self::zero();
        for input in tx.inputs.iter() {
            balance.insert_positive(input.note.unit, input.note.value);
        }
        for (output, _data) in tx.outputs.iter() {
            balance.insert_negative(output.note.unit, output.note.value);
        }

        for MintWitness { unit, amount } in &tx.mints {
            balance.insert_positive(*unit, *amount);
        }

        for BurnWitness { unit, amount } in &tx.burns {
            balance.insert_negative(*unit, *amount);
        }

        balance.clear_zeros();

        balance
    }

    pub fn insert_positive(&mut self, unit: Unit, value: Value) {
        for unit_bal in self.balances.iter_mut() {
            if unit_bal.unit == unit {
                unit_bal.pos += value;
                return;
            }
        }

        // Unit was not found, so we must create one.
        self.balances.push(UnitBalance {
            unit,
            pos: value,
            neg: 0,
        });
    }

    pub fn insert_negative(&mut self, unit: Unit, value: Value) {
        for unit_bal in self.balances.iter_mut() {
            if unit_bal.unit == unit {
                unit_bal.neg += value;
                return;
            }
        }

        self.balances.push(UnitBalance {
            unit,
            pos: 0,
            neg: value,
        });
    }

    pub fn clear_zeros(&mut self) {
        let mut i = 0usize;
        while i < self.balances.len() {
            if self.balances[i].is_zero() {
                self.balances.swap_remove(i);
                // don't increment `i` since the last element has been swapped into the
                // `i`'th place
            } else {
                i += 1;
            }
        }
    }

    pub fn combine<'a>(balances: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut combined = balances
            .into_iter()
            .fold(Balance::zero(), |mut acc, balance| {
                for unit_bal in &balance.balances {
                    if unit_bal.pos > unit_bal.neg {
                        acc.insert_positive(unit_bal.unit, unit_bal.pos - unit_bal.neg);
                    } else {
                        acc.insert_negative(unit_bal.unit, unit_bal.neg - unit_bal.pos);
                    }
                }
                acc
            });
        combined.clear_zeros();
        combined
    }

    pub fn is_zero(&self) -> bool {
        self.balances.is_empty()
    }
}
