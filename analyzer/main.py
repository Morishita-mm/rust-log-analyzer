import redis
import json
import time
import polars as pl

# Redis conf
REDIS_HOST = 'redis'
REDIS_PORT = 6379
LOGS_CHANNEL = 'logs.ingest'
STATS_CHANNEL = 'stats.update'
BUFFER_DURATION_SEC = 1.0   # 集計の間隔


def process_logs(logs_buffer):
    """
    バッファに溜まったログをPolarsで集計し、結果をRedisに送信する
    """
    if not logs_buffer:
        return

    try:
        # 1. 辞書のリストからPolars DataFrameを作成
        df = pl.DataFrame(logs_buffer)
        df = df.with_columns(pl.col("timestamp").str.to_datetime(time_zone="UTC"))
        
        # 3. 集計処理（group_by_dynamicで時間ウィンドウ集計）
        aggregated_df = df.group_by_dynamic("timestamp", every="1s").agg([pl.len().alias("total_count").cast(pl.Int64), # 期間内の総ログ数
                                                                          (pl.col("level") == "ERROR").sum().alias(
                                                                              "error_count").cast(pl.Int64),  # 期間内のエラー回数
                                                                          pl.col("service").mode().first().alias(
                                                                              "top_service")     # 最もログの出力が多かったサービス
                                                                          ])
        aggregated_df = aggregated_df.with_columns([
            pl.col("timestamp").alias("window_start"),
            (pl.col("timestamp") + pl.duration(seconds=1)).alias("window_end")
        ])

        # 不要になった元の 'timestamp' 列を除外し、列の順序を整理します（Rust側の構造体に合わせるため）
        aggregated_df = aggregated_df.select([
            "window_start",
            "window_end",
            "total_count",
            "error_count",
            "top_service"
        ])

        # 4. 集計結果をJSON文字列に変換
        stats_json = aggregated_df.write_json()

        # 5. Redisに送信（Rustが購読しているチャンネルへ）
        # TODO: Redisとの接続は引数で渡す、もしくはグローバルで定義するように変更する
        client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, db=0)
        client.publish(STATS_CHANNEL, stats_json)
        print(f"📊 Sent stats update: {stats_json}")
    except Exception as e:
        print(f"❌ Error processing logs: {e}")


def main():
    try:
        client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, db=0)
        pubsub = client.pubsub()
        pubsub.subscribe(LOGS_CHANNEL)
        client.ping()
        print(f"✅ Connected to Redis. Subscribed to {LOGS_CHANNEL}")
    except redis.ConnectionError as e:
        print(f"❌ Could not connect to Redis: {e}")
        return

    print(f"🚀 Starting dummy log publisher to channel '{LOGS_CHANNEL}'...")

    logs_buffer = []
    last_process_time = time.time()

    try:
        while True:
            message = pubsub.get_message(timeout=0.1)

            if message and message['type'] == 'message':
                try:
                    log_data = json.loads(message['data'])
                    logs_buffer.append(log_data)
                except json.JSONDecodeError:
                    print(f"❌ Received invalid JSON on {LOGS_CHANNEL}")

            current_time = time.time()
            if current_time - last_process_time >= BUFFER_DURATION_SEC:
                if logs_buffer:
                    if logs_buffer:
                        process_logs(logs_buffer)
                        logs_buffer = []
                    last_process_time = current_time

    except KeyboardInterrupt:
        print("\n🔴 Log analyzer stopped.")
        pubsub.close()


if __name__ == "__main__":
    main()
