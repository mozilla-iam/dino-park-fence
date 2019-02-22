use chrono::DateTime;
use chrono::Utc;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Duration;

pub trait RemoteGet {
    fn get(&mut self) -> Result<(), String>;
    fn expiry(&self) -> DateTime<Utc>;
}

pub struct RemoteStore<T: RemoteGet + Default> {
    pub cached: Arc<RwLock<T>>,
    pub is_inflight: Arc<(Mutex<bool>, Condvar)>,
}

impl<T: RemoteGet + Default> Clone for RemoteStore<T> {
    fn clone(&self) -> Self {
        RemoteStore {
            cached: Arc::clone(&self.cached),
            is_inflight: Arc::clone(&self.is_inflight),
        }
    }
}

impl<T: RemoteGet + Default> Default for RemoteStore<T> {
    #[allow(clippy::mutex_atomic)]
    fn default() -> Self {
        RemoteStore {
            cached: Arc::new(RwLock::new(T::default())),
            is_inflight: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}

impl<T: RemoteGet + Default> RemoteStore<T> {
    #[allow(clippy::mutex_atomic)]
    pub fn new(t: T) -> Self {
        RemoteStore {
            cached: Arc::new(RwLock::new(t)),
            is_inflight: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn get(&self) -> Result<Arc<RwLock<T>>, String> {
        let now = Utc::now();
        if let Ok(c) = self.cached.read() {
            if c.expiry() > now {
                return Ok(Arc::clone(&self.cached));
            }
        }
        let inflight_pair = self.is_inflight.clone();
        let &(ref lock, ref cvar) = &*inflight_pair;
        let chosen = {
            let mut is_inflight = lock
                .lock()
                .map_err(|e| format!("Error getting lock for inflight: {}", e))?;
            if !*is_inflight {
                *is_inflight = true;
                true
            } else {
                false
            }
        };
        if chosen {
            let mut k = self
                .cached
                .write()
                .map_err(|e| format!("Unable to write keys: {}", e))?;
            k.get()?;
            let mut is_inflight = lock
                .lock()
                .map_err(|e| format!("Error getting lock for inflight: {}", e))?;
            assert!(*is_inflight);
            *is_inflight = false;
        }
        let mut is_inflight = lock
            .lock()
            .map_err(|e| format!("Error while retrieving remote keys: {}", e))?;
        while *is_inflight {
            let r = cvar
                .wait_timeout(is_inflight, Duration::from_millis(100))
                .map_err(|e| format!("Error while waiting for remote keys: {}", e))?;
            if r.1.timed_out() {
                return Err(String::from("Timeout while waiting for remote keys"));
            }
            is_inflight = r.0;
        }
        Ok(Arc::clone(&self.cached))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;
    use chrono::TimeZone;

    struct R1 {
        pub r: usize,
        pub exp: DateTime<Utc>,
    }

    impl Default for R1 {
        fn default() -> Self {
            R1 {
                r: 0,
                exp: Utc.timestamp(0, 0),
            }
        }
    }

    impl RemoteGet for R1 {
        fn get(&mut self) -> Result<(), String> {
            self.r += 1;
            self.exp = Utc::now();
            Ok(())
        }
        fn expiry(&self) -> DateTime<Utc> {
            self.exp
        }
    }

    struct R2 {
        pub r: usize,
        pub exp: DateTime<Utc>,
    }

    impl Default for R2 {
        fn default() -> Self {
            R2 {
                r: 0,
                exp: Utc.timestamp(0, 0),
            }
        }
    }

    impl RemoteGet for R2 {
        fn get(&mut self) -> Result<(), String> {
            self.r += 1;
            self.exp = Utc::now() + chrono::Duration::seconds(10);
            Ok(())
        }
        fn expiry(&self) -> DateTime<Utc> {
            self.exp
        }
    }

    #[test]
    fn test_get_from_remote() {
        let now = Utc::now();
        let ks = Arc::new(RemoteStore::<R1>::default());
        let ks_c = Arc::clone(&ks);
        let child = thread::spawn(move || {
            let k = ks_c.get();
            assert!(k.is_ok());
            let k = k.unwrap();
            let k = k.read().unwrap();
            assert!(k.expiry() > now);
        });
        {
            let k = ks.get();
            assert!(k.is_ok());
            let k = k.unwrap();
            let k = k.read().unwrap();
            assert!(k.expiry() > now);
        }
        let _ = child.join();
        let k = ks.get();
        assert!(k.is_ok());
        let k = k.unwrap();
        let k = k.read().unwrap();
        assert_eq!(k.r, 3);
    }

    #[test]
    fn test_get_from_remote_cached() {
        let now = Utc::now();
        assert!(Utc::now() + chrono::Duration::seconds(10) > now);
        let ks = Arc::new(RemoteStore::<R2>::default());
        let ks_c = Arc::clone(&ks);
        let child = thread::spawn(move || {
            let k = ks_c.get();
            assert!(k.is_ok());
            let k = k.unwrap();
            let k = k.read().unwrap();
            assert!(k.expiry() > now);
        });
        {
            let k = ks.get();
            assert!(k.is_ok());
            let k = k.unwrap();
            let k = k.read().unwrap();
            assert!(k.expiry() > now);
        }
        let _ = child.join();
        let k = ks.get();
        assert!(k.is_ok());
        let k = k.unwrap();
        let k = k.read().unwrap();
        assert_eq!(k.r, 1);
    }
}
