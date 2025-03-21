use crate::{Digest, Hash};
use serde::{Deserialize, Serialize};

pub type Value = u64;
pub type Unit = [u8; 32];
pub const NOP_COVENANT: [u8; 32] = [0u8; 32];

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct UnitWitness {
    pub spending_covenant: [u8; 32],
    pub minting_covenant: [u8; 32],
    pub burning_covenant: [u8; 32],
    pub arg: [u8; 32],
}

impl UnitWitness {
    pub fn nop(args: &[u8]) -> Self {
        Self {
            spending_covenant: NOP_COVENANT,
            minting_covenant: NOP_COVENANT,
            burning_covenant: NOP_COVENANT,
            arg: crate::hash(args),
        }
    }

    pub fn unit(&self) -> Unit {
        let mut hasher = Hash::new();
        hasher.update(b"NOMOS_CL_UNIT");
        hasher.update(self.spending_covenant);
        hasher.update(self.minting_covenant);
        hasher.update(self.burning_covenant);
        hasher.update(self.arg);
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
    pub fn zero(unit: Unit) -> Self {
        Self {
            unit,
            pos: 0,
            neg: 0,
        }
    }

    pub fn pos(unit: Unit, pos: u64) -> Self {
        Self { unit, pos, neg: 0 }
    }

    pub fn neg(unit: Unit, neg: u64) -> Self {
        Self { unit, pos: 0, neg }
    }

    pub fn is_zero(&self) -> bool {
        self.pos == self.neg
    }

    pub fn is_neg(&self) -> bool {
        self.neg > self.pos
    }

    pub fn is_pos(&self) -> bool {
        self.pos > self.neg
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

    pub fn unit_balance(&self, unit: Unit) -> UnitBalance {
        self.balances
            .iter()
            .find(|b| b.unit == unit)
            .cloned()
            .unwrap_or_else(|| UnitBalance::zero(unit))
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
