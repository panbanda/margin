//! Snooze service for temporarily hiding emails.
//!
//! Allows users to snooze emails until a specific time, hiding them from
//! the inbox until the snooze period expires. Snoozed emails reappear
//! automatically at the scheduled time.

use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, Utc};
use thiserror::Error;

use crate::domain::{AccountId, ThreadId};

/// Errors that can occur during snooze operations.
#[derive(Debug, Error)]
pub enum SnoozeError {
    #[error("Thread not found: {0}")]
    ThreadNotFound(String),

    #[error("Thread is not snoozed: {0}")]
    NotSnoozed(String),

    #[error("Invalid snooze time: {0}")]
    InvalidTime(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

/// Result type for snooze operations.
pub type Result<T> = std::result::Result<T, SnoozeError>;

/// A snoozed email entry.
#[derive(Debug, Clone)]
pub struct SnoozedItem {
    /// Thread that was snoozed.
    pub thread_id: ThreadId,
    /// Account the thread belongs to.
    pub account_id: AccountId,
    /// When the snooze was created.
    pub snoozed_at: DateTime<Utc>,
    /// When the email should reappear.
    pub wake_at: DateTime<Utc>,
    /// Original folder before snoozing.
    pub original_folder: Option<String>,
}

impl SnoozedItem {
    /// Creates a new snoozed item.
    pub fn new(
        thread_id: ThreadId,
        account_id: AccountId,
        wake_at: DateTime<Utc>,
        original_folder: Option<String>,
    ) -> Self {
        Self {
            thread_id,
            account_id,
            snoozed_at: Utc::now(),
            wake_at,
            original_folder,
        }
    }

    /// Returns whether this item should wake up now.
    pub fn should_wake(&self) -> bool {
        Utc::now() >= self.wake_at
    }

    /// Returns the duration until wake time.
    pub fn time_until_wake(&self) -> Duration {
        let now = Utc::now();
        if now >= self.wake_at {
            Duration::zero()
        } else {
            self.wake_at - now
        }
    }
}

/// Common snooze durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnoozeDuration {
    /// Later today (3 hours from now, or 6 PM if evening).
    LaterToday,
    /// Tomorrow at 8 AM.
    Tomorrow,
    /// This weekend (Saturday at 9 AM).
    ThisWeekend,
    /// Next week (Monday at 8 AM).
    NextWeek,
    /// Custom time.
    Custom(DateTime<Utc>),
}

impl SnoozeDuration {
    /// Calculates the wake time for this duration.
    pub fn wake_time(&self) -> DateTime<Utc> {
        let now = Local::now();
        let local_wake = match self {
            SnoozeDuration::LaterToday => {
                let three_hours = now + Duration::hours(3);
                let evening = now
                    .date_naive()
                    .and_time(NaiveTime::from_hms_opt(18, 0, 0).unwrap());
                let evening_dt = evening.and_local_timezone(now.timezone()).unwrap();

                if three_hours.time() >= NaiveTime::from_hms_opt(18, 0, 0).unwrap() {
                    // If 3 hours from now is past 6 PM, use tomorrow morning
                    let tomorrow = now.date_naive() + Duration::days(1);
                    let morning = tomorrow.and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
                    morning.and_local_timezone(now.timezone()).unwrap()
                } else if now.time() >= NaiveTime::from_hms_opt(18, 0, 0).unwrap() {
                    // Already evening, use tomorrow
                    let tomorrow = now.date_naive() + Duration::days(1);
                    let morning = tomorrow.and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
                    morning.and_local_timezone(now.timezone()).unwrap()
                } else {
                    // Use 6 PM or 3 hours, whichever is later
                    std::cmp::max(three_hours, evening_dt)
                }
            }
            SnoozeDuration::Tomorrow => {
                let tomorrow = now.date_naive() + Duration::days(1);
                let morning = tomorrow.and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
                morning.and_local_timezone(now.timezone()).unwrap()
            }
            SnoozeDuration::ThisWeekend => {
                let weekday = now.weekday().num_days_from_monday();
                let days_until_saturday = if weekday >= 5 {
                    // Already weekend, use next Saturday
                    7 - weekday + 5
                } else {
                    5 - weekday
                };
                let saturday = now.date_naive() + Duration::days(days_until_saturday as i64);
                let morning = saturday.and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap());
                morning.and_local_timezone(now.timezone()).unwrap()
            }
            SnoozeDuration::NextWeek => {
                let weekday = now.weekday().num_days_from_monday();
                let days_until_monday = 7 - weekday;
                let monday = now.date_naive() + Duration::days(days_until_monday as i64);
                let morning = monday.and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
                morning.and_local_timezone(now.timezone()).unwrap()
            }
            SnoozeDuration::Custom(dt) => return *dt,
        };

        local_wake.with_timezone(&Utc)
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> String {
        match self {
            SnoozeDuration::LaterToday => "Later today".to_string(),
            SnoozeDuration::Tomorrow => "Tomorrow morning".to_string(),
            SnoozeDuration::ThisWeekend => "This weekend".to_string(),
            SnoozeDuration::NextWeek => "Next week".to_string(),
            SnoozeDuration::Custom(dt) => {
                let local = dt.with_timezone(&Local);
                local.format("%a, %b %d at %I:%M %p").to_string()
            }
        }
    }
}

/// Storage trait for persisting snooze data.
pub trait SnoozeStorage: Send + Sync {
    /// Stores a snoozed item.
    fn store_snooze(&self, item: &SnoozedItem) -> Result<()>;

    /// Removes a snooze entry.
    fn remove_snooze(&self, thread_id: &ThreadId) -> Result<()>;

    /// Gets a snoozed item by thread ID.
    fn get_snooze(&self, thread_id: &ThreadId) -> Result<Option<SnoozedItem>>;

    /// Gets all snoozed items for an account.
    fn get_snoozed_for_account(&self, account_id: &AccountId) -> Result<Vec<SnoozedItem>>;

    /// Gets all items that should wake up now.
    fn get_items_to_wake(&self) -> Result<Vec<SnoozedItem>>;

    /// Gets all snoozed items.
    fn get_all_snoozed(&self) -> Result<Vec<SnoozedItem>>;
}

/// Service for managing email snooze functionality.
pub struct SnoozeService<S: SnoozeStorage> {
    storage: S,
}

impl<S: SnoozeStorage> SnoozeService<S> {
    /// Creates a new snooze service.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Snoozes a thread until the specified time.
    pub fn snooze(
        &self,
        thread_id: ThreadId,
        account_id: AccountId,
        duration: SnoozeDuration,
        original_folder: Option<String>,
    ) -> Result<SnoozedItem> {
        let wake_at = duration.wake_time();

        if wake_at <= Utc::now() {
            return Err(SnoozeError::InvalidTime(
                "Wake time must be in the future".to_string(),
            ));
        }

        let item = SnoozedItem::new(thread_id, account_id, wake_at, original_folder);
        self.storage.store_snooze(&item)?;
        Ok(item)
    }

    /// Snoozes a thread until a specific datetime.
    pub fn snooze_until(
        &self,
        thread_id: ThreadId,
        account_id: AccountId,
        wake_at: DateTime<Utc>,
        original_folder: Option<String>,
    ) -> Result<SnoozedItem> {
        self.snooze(
            thread_id,
            account_id,
            SnoozeDuration::Custom(wake_at),
            original_folder,
        )
    }

    /// Unsnoozes a thread, making it immediately visible.
    pub fn unsnooze(&self, thread_id: &ThreadId) -> Result<Option<SnoozedItem>> {
        let item = self.storage.get_snooze(thread_id)?;
        if item.is_some() {
            self.storage.remove_snooze(thread_id)?;
        }
        Ok(item)
    }

    /// Gets the snooze status for a thread.
    pub fn get_snooze(&self, thread_id: &ThreadId) -> Result<Option<SnoozedItem>> {
        self.storage.get_snooze(thread_id)
    }

    /// Checks if a thread is snoozed.
    pub fn is_snoozed(&self, thread_id: &ThreadId) -> Result<bool> {
        Ok(self.storage.get_snooze(thread_id)?.is_some())
    }

    /// Gets all snoozed items for an account.
    pub fn get_snoozed_for_account(&self, account_id: &AccountId) -> Result<Vec<SnoozedItem>> {
        self.storage.get_snoozed_for_account(account_id)
    }

    /// Gets all items that should wake up now.
    pub fn get_items_to_wake(&self) -> Result<Vec<SnoozedItem>> {
        self.storage.get_items_to_wake()
    }

    /// Processes all items that should wake up, returning them and removing from storage.
    pub fn process_wakeups(&self) -> Result<Vec<SnoozedItem>> {
        let items = self.storage.get_items_to_wake()?;
        for item in &items {
            self.storage.remove_snooze(&item.thread_id)?;
        }
        Ok(items)
    }

    /// Gets count of snoozed items.
    pub fn snoozed_count(&self) -> Result<usize> {
        Ok(self.storage.get_all_snoozed()?.len())
    }

    /// Updates the wake time for a snoozed item.
    pub fn update_snooze(
        &self,
        thread_id: &ThreadId,
        new_duration: SnoozeDuration,
    ) -> Result<SnoozedItem> {
        let existing = self
            .storage
            .get_snooze(thread_id)?
            .ok_or_else(|| SnoozeError::NotSnoozed(thread_id.to_string()))?;

        let wake_at = new_duration.wake_time();
        if wake_at <= Utc::now() {
            return Err(SnoozeError::InvalidTime(
                "Wake time must be in the future".to_string(),
            ));
        }

        let updated = SnoozedItem {
            wake_at,
            ..existing
        };
        self.storage.store_snooze(&updated)?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// In-memory storage for testing.
    struct MockStorage {
        items: RwLock<HashMap<String, SnoozedItem>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                items: RwLock::new(HashMap::new()),
            }
        }
    }

    impl SnoozeStorage for MockStorage {
        fn store_snooze(&self, item: &SnoozedItem) -> Result<()> {
            self.items
                .write()
                .unwrap()
                .insert(item.thread_id.to_string(), item.clone());
            Ok(())
        }

        fn remove_snooze(&self, thread_id: &ThreadId) -> Result<()> {
            self.items.write().unwrap().remove(&thread_id.to_string());
            Ok(())
        }

        fn get_snooze(&self, thread_id: &ThreadId) -> Result<Option<SnoozedItem>> {
            Ok(self
                .items
                .read()
                .unwrap()
                .get(&thread_id.to_string())
                .cloned())
        }

        fn get_snoozed_for_account(&self, account_id: &AccountId) -> Result<Vec<SnoozedItem>> {
            Ok(self
                .items
                .read()
                .unwrap()
                .values()
                .filter(|i| i.account_id == *account_id)
                .cloned()
                .collect())
        }

        fn get_items_to_wake(&self) -> Result<Vec<SnoozedItem>> {
            let now = Utc::now();
            Ok(self
                .items
                .read()
                .unwrap()
                .values()
                .filter(|i| i.wake_at <= now)
                .cloned()
                .collect())
        }

        fn get_all_snoozed(&self) -> Result<Vec<SnoozedItem>> {
            Ok(self.items.read().unwrap().values().cloned().collect())
        }
    }

    fn make_thread_id(s: &str) -> ThreadId {
        ThreadId::from(s.to_string())
    }

    fn make_account_id(s: &str) -> AccountId {
        AccountId::from(s.to_string())
    }

    #[test]
    fn snooze_duration_descriptions() {
        assert_eq!(SnoozeDuration::LaterToday.description(), "Later today");
        assert_eq!(SnoozeDuration::Tomorrow.description(), "Tomorrow morning");
        assert_eq!(SnoozeDuration::ThisWeekend.description(), "This weekend");
        assert_eq!(SnoozeDuration::NextWeek.description(), "Next week");
    }

    #[test]
    fn snooze_duration_wake_times_are_future() {
        let durations = [
            SnoozeDuration::LaterToday,
            SnoozeDuration::Tomorrow,
            SnoozeDuration::ThisWeekend,
            SnoozeDuration::NextWeek,
        ];

        let now = Utc::now();
        for duration in durations {
            let wake = duration.wake_time();
            assert!(wake > now, "{:?} wake time should be in the future", duration);
        }
    }

    #[test]
    fn snooze_and_unsnooze() {
        let storage = MockStorage::new();
        let service = SnoozeService::new(storage);

        let thread_id = make_thread_id("thread-1");
        let account_id = make_account_id("account-1");

        let wake_at = Utc::now() + Duration::hours(1);
        let item = service
            .snooze_until(thread_id.clone(), account_id, wake_at, Some("INBOX".to_string()))
            .unwrap();

        assert!(service.is_snoozed(&thread_id).unwrap());
        assert_eq!(item.original_folder, Some("INBOX".to_string()));

        let unsnooozed = service.unsnooze(&thread_id).unwrap();
        assert!(unsnooozed.is_some());
        assert!(!service.is_snoozed(&thread_id).unwrap());
    }

    #[test]
    fn snooze_past_time_fails() {
        let storage = MockStorage::new();
        let service = SnoozeService::new(storage);

        let wake_at = Utc::now() - Duration::hours(1);
        let result = service.snooze_until(
            make_thread_id("thread-1"),
            make_account_id("account-1"),
            wake_at,
            None,
        );

        assert!(matches!(result, Err(SnoozeError::InvalidTime(_))));
    }

    #[test]
    fn process_wakeups() {
        let storage = MockStorage::new();
        let service = SnoozeService::new(storage);

        // Snooze one for the past (should wake) and one for the future
        let past_wake = Utc::now() - Duration::seconds(1);
        let future_wake = Utc::now() + Duration::hours(1);

        // Manually insert past item to bypass validation
        {
            let item = SnoozedItem {
                thread_id: make_thread_id("thread-past"),
                account_id: make_account_id("account-1"),
                snoozed_at: Utc::now() - Duration::hours(1),
                wake_at: past_wake,
                original_folder: None,
            };
            service.storage.store_snooze(&item).unwrap();
        }

        service
            .snooze_until(
                make_thread_id("thread-future"),
                make_account_id("account-1"),
                future_wake,
                None,
            )
            .unwrap();

        let woken = service.process_wakeups().unwrap();
        assert_eq!(woken.len(), 1);
        assert_eq!(woken[0].thread_id, make_thread_id("thread-past"));

        // Past item should be removed, future should remain
        assert!(!service.is_snoozed(&make_thread_id("thread-past")).unwrap());
        assert!(service.is_snoozed(&make_thread_id("thread-future")).unwrap());
    }

    #[test]
    fn update_snooze_time() {
        let storage = MockStorage::new();
        let service = SnoozeService::new(storage);

        let thread_id = make_thread_id("thread-1");
        let wake_at = Utc::now() + Duration::hours(1);

        service
            .snooze_until(thread_id.clone(), make_account_id("account-1"), wake_at, None)
            .unwrap();

        let updated = service
            .update_snooze(&thread_id, SnoozeDuration::Tomorrow)
            .unwrap();

        assert!(updated.wake_at > wake_at);
    }

    #[test]
    fn update_non_snoozed_fails() {
        let storage = MockStorage::new();
        let service = SnoozeService::new(storage);

        let result = service.update_snooze(&make_thread_id("nonexistent"), SnoozeDuration::Tomorrow);
        assert!(matches!(result, Err(SnoozeError::NotSnoozed(_))));
    }

    #[test]
    fn get_snoozed_for_account() {
        let storage = MockStorage::new();
        let service = SnoozeService::new(storage);

        let account1 = make_account_id("account-1");
        let account2 = make_account_id("account-2");
        let wake_at = Utc::now() + Duration::hours(1);

        service
            .snooze_until(make_thread_id("t1"), account1.clone(), wake_at, None)
            .unwrap();
        service
            .snooze_until(make_thread_id("t2"), account1.clone(), wake_at, None)
            .unwrap();
        service
            .snooze_until(make_thread_id("t3"), account2.clone(), wake_at, None)
            .unwrap();

        let account1_snoozed = service.get_snoozed_for_account(&account1).unwrap();
        let account2_snoozed = service.get_snoozed_for_account(&account2).unwrap();

        assert_eq!(account1_snoozed.len(), 2);
        assert_eq!(account2_snoozed.len(), 1);
    }

    #[test]
    fn snoozed_item_should_wake() {
        let past = SnoozedItem {
            thread_id: make_thread_id("t1"),
            account_id: make_account_id("a1"),
            snoozed_at: Utc::now() - Duration::hours(2),
            wake_at: Utc::now() - Duration::hours(1),
            original_folder: None,
        };

        let future = SnoozedItem {
            thread_id: make_thread_id("t2"),
            account_id: make_account_id("a1"),
            snoozed_at: Utc::now(),
            wake_at: Utc::now() + Duration::hours(1),
            original_folder: None,
        };

        assert!(past.should_wake());
        assert!(!future.should_wake());
    }

    #[test]
    fn time_until_wake() {
        let item = SnoozedItem {
            thread_id: make_thread_id("t1"),
            account_id: make_account_id("a1"),
            snoozed_at: Utc::now(),
            wake_at: Utc::now() + Duration::hours(2),
            original_folder: None,
        };

        let time_left = item.time_until_wake();
        assert!(time_left > Duration::hours(1));
        assert!(time_left <= Duration::hours(2));
    }
}
