#[macro_use]
extern crate serde;

use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

#[allow(dead_code)]
type Memory = VirtualMemory<DefaultMemoryImpl>;
#[allow(dead_code)]
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct CalculatorResult {
    result: f64,
}

impl Storable for CalculatorResult {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for CalculatorResult {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, CalculatorResult, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));
}

#[derive(candid::CandidType, Serialize, Deserialize)]
enum CalculatorOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Power,
}

// Default trait for CalculatorOperation
impl Default for CalculatorOperation {
    fn default() -> Self {
        CalculatorOperation::Add
    }
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct CalculatorPayload {
    num1: f64,
    num2: f64,
    operation: CalculatorOperation,
}

#[ic_cdk::query]
fn calculate(payload: CalculatorPayload) -> Result<CalculatorResult, Error> {
    let result = match payload.operation {
        CalculatorOperation::Add => add(payload.num1, payload.num2),
        CalculatorOperation::Subtract => subtract(payload.num1, payload.num2),
        CalculatorOperation::Multiply => multiply(payload.num1, payload.num2),
        CalculatorOperation::Divide => divide(payload.num1, payload.num2)?,
        CalculatorOperation::Mod => modulus(payload.num1, payload.num2),
        CalculatorOperation::Power => power(payload.num1, payload.num2),
    };

    Ok(CalculatorResult { result })
}

fn add(num1: f64, num2: f64) -> f64 {
    num1 + num2
}

fn subtract(num1: f64, num2: f64) -> f64 {
    num1 - num2
}

fn multiply(num1: f64, num2: f64) -> f64 {
    num1 * num2
}

fn divide(num1: f64, num2: f64) -> Result<f64, Error> {
    if num2 == 0.0 {
        Err(Error::DivisionByZero)
    } else {
        Ok(num1 / num2)
    }
}

fn modulus(num1: f64, num2: f64) -> f64 {
    num1 % num2
}

fn power(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    DivisionByZero,
}

ic_cdk::export_candid!();
