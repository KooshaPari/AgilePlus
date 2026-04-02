use phenotype_cache_adapter::TwoTierCache;

fn main() {
    let cache = TwoTierCache::new(100, 1000);

    cache.put("key1".to_string(), "value1".to_string());
    if let Some(value) = cache.get(&"key1".to_string()) {
        println!("Got value: {}", value);
    }
}
