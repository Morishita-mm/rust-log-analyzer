from flask import Flask, request
import logging
import sys
import random
import time
import datetime
from pythonjsonlogger import jsonlogger

# --- ロガーの設定 ---
# 標準出力(stdout)にJSON形式でログを出すように設定
logger = logging.getLogger()
logHandler = logging.StreamHandler(sys.stdout)

class CustomJsonFormatter(jsonlogger.JsonFormatter):
    def add_fields(self, log_record, record, message_dict):
        super(CustomJsonFormatter, self).add_fields(log_record, record, message_dict)
        if not log_record.get('timestamp'):
            # 現在時刻をUTCで取得し、指定のフォーマットで文字列化
            now = datetime.datetime.utcnow()
            log_record['timestamp'] = now.strftime('%Y-%m-%dT%H:%M:%S.%fZ')

formatter = CustomJsonFormatter(
    '%(levelname)s %(service)s %(message)s',
    rename_fields={'levelname': 'level'} # levelnameをlevelにリネームするとより一般的
)

logHandler.setFormatter(formatter)
logger.addHandler(logHandler)
logger.setLevel(logging.INFO)

# サービス名を固定
SERVICE_NAME = "payment-api"

app = Flask(__name__)

# カスタムフィールド（service名）をログに付与するためのフィルター
class ServiceContextFilter(logging.Filter):
    def filter(self, record):
        record.service = SERVICE_NAME
        return True


logger.addFilter(ServiceContextFilter())

# --- ルート定義 ---

@app.route("/")
def index():
    # 正常なアクセスログ (INFO)
    logger.info("Processed request for / endpoint",
                extra={"path": "/", "method": request.method})

    # ランダムにエラーを発生させるシミュレーション
    if random.random() < 0.2:
        try:
            # 意図的な例外発生
            1 / 0
        except ZeroDivisionError:
            # エラーログ (ERROR) - スタックトレースもJSONに含まれます
            logger.error(
                "Database connection failed due to zero division simulation", exc_info=True)
            return "Internal Server Error", 500

    return "Hello, this is a sample app producing JSON logs!", 200


@app.route("/checkout")
def checkout():
    logger.info("Processing checkout", extra={"user_id": random.randint(
        100, 999), "amount": random.randint(10, 5000)})
    time.sleep(random.uniform(0.1, 0.5))  # 少し遅延させる
    return "Checkout complete", 200


if __name__ == "__main__":
    # コンテナ内で実行するためホストを0.0.0.0に設定
    app.run(host="0.0.0.0", port=5001)