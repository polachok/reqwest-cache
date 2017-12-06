extern crate reqwest;
extern crate lru_time_cache;
extern crate serde;

#[cfg(test)]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

use std::hash::Hash;
use std::time::Duration;
use reqwest::Client;
use reqwest::IntoUrl;
use lru_time_cache::LruCache;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct CachingClient<T> {
	client: Client,
	cache: LruCache<(u64, u64), T>,
}

fn do_hash<H: Hash>(v: &H) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

impl<T: DeserializeOwned + Clone> CachingClient<T> {
	pub fn with_capacity(cache_size: usize) -> Self {
		CachingClient {
			client: Client::new(),
			cache: LruCache::with_capacity(cache_size),
		}
	}

    pub fn with_expiry_duration(duration: Duration) -> Self {
        CachingClient {
            client: Client::new(),
            cache: LruCache::with_expiry_duration(duration),
        }
    }


    pub fn with_expiry_duration_and_capacity(cache_size: usize, duration: Duration) -> Self {
        CachingClient {
            client: Client::new(),
            cache: LruCache::with_expiry_duration_and_capacity(duration, cache_size),
        }
    }

    pub fn post_json<U: IntoUrl + Hash, S: Serialize + Hash>(&mut self, url: U, body: &S) -> Result<T, reqwest::Error> {
        use lru_time_cache::Entry;
        let body_hash = do_hash(body);
        let url_hash = do_hash(&url);
        let entry = self.cache.entry((url_hash, body_hash));
        let result = match entry {
            Entry::Vacant(entry) => {
                let res: T = self.client.post(url).json(&body).send()?.json()?;
                entry.insert(res.clone());
                res
            }
            Entry::Occupied(entry) => {
                entry.into_mut().clone()
            }
        };
        Ok(result)
    }
}