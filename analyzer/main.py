import redis
import json
import time
import datetime

def main():
    client = redis.Redis(host='redis', port=6379, db=0)
    
    print("🚀 Python Log Publisher started...")
    
    while True:
        # ダミーログデータ
        log_entry = {
            "timestamp": datetime.datetime.now().isoformat(),
            "level": "INFO",
            "service": "auth-service",
            "message": "User login successful"
        }
        
        # JSONに変換して'logs.ingest'チャンネルに送信
        message = json.dumps(log_entry)
        client.publish('logs.ingest', message)
        
        print(f"Send: {message}")
        time.sleep(1)

if __name__ == "__main__":
    main()
