//! Companion library for the lesson **Testing & Documentation**.
//!
//! It is a tiny bank-account library, chosen because it has a happy path, error
//! cases, and an invariant worth protecting — exactly what tests are for.
//!
//! Running `cargo test` here exercises three kinds of test at once:
//! unit tests (this file), a doc-test (the example below), and an integration
//! test (in `tests/`).

/// Adds two balances. Kept trivial so the **doc-test** is the star: the example
/// below is compiled and run by `cargo test`, so the docs can never drift from
/// the code.
///
/// # Examples
///
/// ```
/// use testing::add;
/// assert_eq!(add(2, 40), 42);
/// ```
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// The error type for account operations.
#[derive(Debug, PartialEq, Eq)]
pub enum BankError {
    /// Tried to withdraw or deposit a non-positive amount.
    NonPositiveAmount,
    /// Tried to withdraw more than the available balance.
    InsufficientFunds { balance: u64, requested: u64 },
}

/// A minimal bank account whose balance can never go negative — an invariant the
/// tests below pin down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    balance: u64,
}

impl Account {
    /// Opens an account with an initial balance.
    ///
    /// # Examples
    ///
    /// ```
    /// use testing::Account;
    /// let acc = Account::new(100);
    /// assert_eq!(acc.balance(), 100);
    /// ```
    pub fn new(initial: u64) -> Self {
        Account { balance: initial }
    }

    /// The current balance.
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Deposits `amount`, returning the new balance or an error.
    pub fn deposit(&mut self, amount: u64) -> Result<u64, BankError> {
        if amount == 0 {
            return Err(BankError::NonPositiveAmount);
        }
        self.balance += amount;
        Ok(self.balance)
    }

    /// Withdraws `amount`, returning the new balance or an error. Never lets the
    /// balance go negative.
    pub fn withdraw(&mut self, amount: u64) -> Result<u64, BankError> {
        if amount == 0 {
            return Err(BankError::NonPositiveAmount);
        }
        if amount > self.balance {
            return Err(BankError::InsufficientFunds {
                balance: self.balance,
                requested: amount,
            });
        }
        self.balance -= amount;
        Ok(self.balance)
    }
}

/// Divides `a` by `b`. **Panics** if `b == 0` — used in the lesson to show
/// `#[should_panic]` testing.
///
/// # Panics
///
/// Panics if `b` is zero.
pub fn divide(a: i64, b: i64) -> i64 {
    if b == 0 {
        panic!("division by zero");
    }
    a / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sums_two_numbers() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-5, 5), 0);
    }

    #[test]
    fn deposit_increases_balance() {
        let mut acc = Account::new(100);
        assert_eq!(acc.deposit(50), Ok(150));
        assert_eq!(acc.balance(), 150);
    }

    #[test]
    fn deposit_rejects_zero() {
        let mut acc = Account::new(100);
        assert_eq!(acc.deposit(0), Err(BankError::NonPositiveAmount));
        assert_eq!(acc.balance(), 100); // unchanged
    }

    #[test]
    fn withdraw_reduces_balance() {
        let mut acc = Account::new(100);
        assert_eq!(acc.withdraw(30), Ok(70));
    }

    #[test]
    fn withdraw_cannot_overdraw() {
        let mut acc = Account::new(100);
        assert_eq!(
            acc.withdraw(150),
            Err(BankError::InsufficientFunds {
                balance: 100,
                requested: 150
            })
        );
        // The invariant held: balance is untouched after a failed withdrawal.
        assert_eq!(acc.balance(), 100);
    }

    #[test]
    fn divide_works_for_nonzero() {
        assert_eq!(divide(84, 2), 42);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn divide_by_zero_panics() {
        divide(1, 0);
    }

    // A test that returns Result can use `?` internally — handy for chains of
    // fallible steps. It passes if it returns Ok(()).
    #[test]
    fn deposit_then_withdraw_roundtrips() -> Result<(), BankError> {
        let mut acc = Account::new(0);
        acc.deposit(100)?;
        acc.withdraw(40)?;
        assert_eq!(acc.balance(), 60);
        Ok(())
    }
}
