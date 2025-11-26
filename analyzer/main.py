import redis
import json
import time
import datetime
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

        # 2. timestamp文字列をdatetime型にへんかn
        df = df.with_columns(pl.col("timestamp").str.to_datetime())

        # 3. 集計処理（group_by_dynamicで時間ウィンドウ集計）
        aggregated_df = df.group_by_dynamic("timestamp", every="1s").agg([pl.len().alias("total_count"),                            # 期間内の総ログ数
                                                                          (pl.col("level") == "ERROR").sum().alias("error_count"),  # 期間内のエラー回数
                                                                          pl.col("service").mode().first().alias("top_service")     # 最もログの出力が多かったサービス
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
    client = redis.Redis(host='redis', port=6379, db=0)
    pubsub = client.pubsub()
    pubsub.subscribe(LOGS_CHANNEL)

    print("🚀 Python Log Publisher started. Listening on '{LOGS_CHANNEL}'...")
    
    logs_buffer = []
    last_process_time = time.time()
    
    # ダミーログを別スレッドで送信する仕組みが必要だが、動作定義のため、受信ループの中で擬似的にログを生成してバッファに追加します
    # TODO: 別スレッドで実施するように変更する

    while True:
        # --- 擬似的なログ生成（動作確認用）---
        # 実際にはRedisからのメッセージ受信のみになる
        dummy_log = {
            "timestamp": datetime.datetime.now().isoformat(),
            "level": "INFO" if time.time() % 2 > 0.5 else "ERROR",  # ランダムにERRORにする
            "service": "auth-service",
            "message": "User login successful"
        }
        logs_buffer.append(dummy_log)
        time.sleep(0.1) # 0.1秒に1件ログが発生すると仮定
        
        # TODO: 本来はメッセージ受信ループがここに入る
        # message = pubsub.get_message()
        # if message and message['type'] == 'message':
        #     try:
        #         log_data = json.loads(message['data'])
        #         logs_buffer.append(log_data)
        #     except json.JSONDecodeError:
        #         print("❌ Received invalid JSON")

        # 一定時間経過したらバッファを処理
        current_time = time.time()
        if current_time - last_process_time >= BUFFER_DURATION_SEC:
            process_logs(logs_buffer)
            logs_buffer = []   # バッファをクリア
            last_process_time = current_time

if __name__ == "__main__":
    main()
