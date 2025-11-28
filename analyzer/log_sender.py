import redis
import json
import time
import datetime
import random

# Redis conf
REDIS_HOST = 'redis'
REDIS_PORT = 6379
LOGS_CHANNEL = 'logs.ingest'
STATS_CHANNEL = 'stats.update'

def generate_dummy_log():
    """ダミーのログエントリを生成"""
    services = ["auth-service", "payment-service", "user-service", "db-service"]
    levels = ["INFO", "WARN", "ERROR", "DEBUG"]
    messages = [
        "User login successful",
        "Failed to connect to databse",
        "Payment processed successfully",
        "Cache miss for user profile",
        "API rate limit exceeded",
        "User logged out",
    ]
    
    level = "ERROR" if random.random() > 0.8 else random.choice(levels)
    
    return {
        "timestamp": datetime.datetime.now().isoformat(),
        "level": level,
        "service": random.choice(services),
        "message": random.choice(messages),
    }
    
def main():
    try:
        client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, db=0)
        client.ping()
        print(f"✅ Connected to Redis at {REDIS_HOST}:{REDIS_PORT}")
    except redis.ConnectionError as e:
        print(f"❌ Could not connect to Redis: {e}")
        return
    
    print(f"🚀 Starting dummy log publisher to channel '{LOGS_CHANNEL}'...")
    print("\tPress Ctrl+C to stop.")
    
    try:
        while True:
            log_entry = generate_dummy_log()
            log_json = json.dumps(log_entry)
            
            client.publish(LOGS_CHANNEL, log_json)
            
            print(f"-> Sent: {log_json}")
            
            time.sleep(random.uniform(0.05, 0.3))
    except KeyboardInterrupt:
        print("\n🔴 Log publisher stopped.")

if __name__ == "__main__":
    main()