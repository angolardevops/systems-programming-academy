//! Integration test: lives in `tests/`, so it is compiled as a **separate
//! crate** that uses `testing` exactly as an external user would — only the
//! public API is visible here. This is the difference from unit tests, which
//! live inside the library and can see private items.

use testing::{Account, BankError};

#[test]
fn a_full_customer_session() {
    let mut acc = Account::new(0);

    // Deposit twice, withdraw once, and check the running balance.
    assert_eq!(acc.deposit(200), Ok(200));
    assert_eq!(acc.deposit(50), Ok(250));
    assert_eq!(acc.withdraw(75), Ok(175));
    assert_eq!(acc.balance(), 175);

    // Overdraw is rejected and leaves the balance intact.
    assert_eq!(
        acc.withdraw(1_000),
        Err(BankError::InsufficientFunds {
            balance: 175,
            requested: 1_000
        })
    );
    assert_eq!(acc.balance(), 175);
}
