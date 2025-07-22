#!/usr/bin/env python3
"""
BingX Position Manager
Автоматическое управление позициями на BingX:
- Проверка наличия stop loss у всех открытых позиций
- Добавление stop loss если отсутствует
- Частичное закрытие позиций при достижении 1% прибыли (без учета плеча)
- Перевод stop loss в безубыток
"""

import json
import logging
import time
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple

try:
    import requests
except ImportError:
    print("Ошибка: Не установлена библиотека requests")
    print("Установите её командой: pip install requests")
    sys.exit(1)

# Попробуем импортировать bingx, если не получится - используем requests напрямую
try:
    from bingx.api import BingxAPI
    BINGX_LIB_AVAILABLE = True
except ImportError:
    print("Библиотека bingx-py не найдена, используем прямые API вызовы")
    BINGX_LIB_AVAILABLE = False

# Конфигурация
CONFIG_FILE = "bingx_config.json"
LOG_FILE = "bingx_position_manager.log"
CHECK_INTERVAL = 60  # секунд
PROFIT_THRESHOLD = 0.01  # 1% прибыли без учета плеча
PARTIAL_CLOSE_PERCENT = 0.5  # 50% от позиции

class BingXAPIClient:
    """Клиент для работы с BingX API через requests"""

    def __init__(self, api_key: str, secret_key: str, testnet: bool = True):
        self.api_key = api_key
        self.secret_key = secret_key
        self.base_url = "https://open-api-vst.bingx.com" if testnet else "https://open-api.bingx.com"

    def _sign_request(self, params: Dict) -> str:
        """Подпись запроса"""
        import hmac
        import hashlib

        query_string = "&".join([f"{k}={v}" for k, v in sorted(params.items())])
        signature = hmac.new(
            self.secret_key.encode('utf-8'),
            query_string.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()
        return signature

    def _make_request(self, method: str, endpoint: str, params: Dict = None) -> Dict:
        """Выполнение запроса к API"""
        if params is None:
            params = {}

        params['timestamp'] = int(time.time() * 1000)
        params['signature'] = self._sign_request(params)

        headers = {
            'X-BX-APIKEY': self.api_key,
            'Content-Type': 'application/json'
        }

        url = f"{self.base_url}{endpoint}"

        try:
            if method == 'GET':
                response = requests.get(url, params=params, headers=headers, timeout=10)
            elif method == 'POST':
                response = requests.post(url, json=params, headers=headers, timeout=10)
            elif method == 'DELETE':
                response = requests.delete(url, params=params, headers=headers, timeout=10)
            else:
                raise ValueError(f"Неподдерживаемый метод: {method}")

            return response.json()
        except Exception as e:
            return {'code': -1, 'msg': str(e)}

    def get_positions(self) -> Dict:
        """Получение позиций"""
        return self._make_request('GET', '/fapi/v1/balance')

    def get_open_orders(self, symbol: str) -> Dict:
        """Получение открытых ордеров"""
        params = {'symbol': symbol}
        return self._make_request('GET', '/fapi/v1/openOrders', params)

    def place_order(self, symbol: str, side: str, type: str, quantity: float,
                   price: float = None, stopPrice: float = None, **kwargs) -> Dict:
        """Размещение ордера"""
        params = {
            'symbol': symbol,
            'side': side,
            'type': type,
            'quantity': quantity
        }

        if price:
            params['price'] = price
        if stopPrice:
            params['stopPrice'] = stopPrice

        params.update(kwargs)
        return self._make_request('POST', '/fapi/v1/order', params)

    def cancel_order(self, symbol: str, orderId: str) -> Dict:
        """Отмена ордера"""
        params = {'symbol': symbol, 'orderId': orderId}
        return self._make_request('DELETE', '/fapi/v1/order', params)

class BingXPositionManager:
    def __init__(self, config_path: str = CONFIG_FILE):
        self.config_path = config_path
        self.config = self.load_config()
        self.api = self.init_api()
        self.setup_logging()
        
    def load_config(self) -> Dict:
        """Загрузка конфигурации из файла"""
        if not os.path.exists(self.config_path):
            self.create_default_config()
            
        try:
            with open(self.config_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"Ошибка загрузки конфигурации: {e}")
            sys.exit(1)
            
    def create_default_config(self):
        """Создание файла конфигурации по умолчанию"""
        default_config = {
            "api_key": "YOUR_API_KEY",
            "secret_key": "YOUR_SECRET_KEY",
            "testnet": True,
            "check_interval": CHECK_INTERVAL,
            "profit_threshold": PROFIT_THRESHOLD,
            "partial_close_percent": PARTIAL_CLOSE_PERCENT,
            "stop_loss_offset": 0.005,  # 0.5% от цены входа для stop loss
            "min_position_size": 10,  # минимальный размер позиции в USDT
            "enabled_symbols": [],  # пустой список = все символы
            "disabled_symbols": ["BTCUSDT"]  # исключенные символы
        }
        
        with open(self.config_path, 'w', encoding='utf-8') as f:
            json.dump(default_config, f, indent=2, ensure_ascii=False)
            
        print(f"Создан файл конфигурации: {self.config_path}")
        print("Пожалуйста, заполните API ключи и настройки перед запуском")
        sys.exit(0)
        
    def init_api(self):
        """Инициализация API клиента BingX"""
        try:
            if BINGX_LIB_AVAILABLE:
                api = BingxAPI(
                    api_key=self.config['api_key'],
                    secret_key=self.config['secret_key'],
                    testnet=self.config.get('testnet', True)
                )
            else:
                api = BingXAPIClient(
                    api_key=self.config['api_key'],
                    secret_key=self.config['secret_key'],
                    testnet=self.config.get('testnet', True)
                )
            return api
        except Exception as e:
            print(f"Ошибка инициализации API: {e}")
            sys.exit(1)
            
    def setup_logging(self):
        """Настройка логирования"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(LOG_FILE, encoding='utf-8'),
                logging.StreamHandler(sys.stdout)
            ]
        )
        self.logger = logging.getLogger(__name__)
        self.daily_trades = 0
        self.error_count = 0
        self.last_reset_date = datetime.now().date()

    def reset_daily_counters(self):
        """Сброс дневных счетчиков"""
        current_date = datetime.now().date()
        if current_date != self.last_reset_date:
            self.daily_trades = 0
            self.error_count = 0
            self.last_reset_date = current_date
            self.logger.info("Сброшены дневные счетчики")

    def check_emergency_stop(self) -> bool:
        """Проверка аварийной остановки"""
        if self.config.get('emergency_stop', False):
            self.logger.warning("Активирована аварийная остановка!")
            return True

        # Проверка лимитов
        risk_config = self.config.get('risk_management', {})
        max_daily_trades = risk_config.get('max_daily_trades', 50)
        error_threshold = risk_config.get('error_threshold', 5)

        if self.daily_trades >= max_daily_trades:
            self.logger.warning(f"Достигнут лимит дневных сделок: {max_daily_trades}")
            return True

        if self.error_count >= error_threshold:
            self.logger.warning(f"Достигнут лимит ошибок: {error_threshold}")
            return True

        return False

    def send_notification(self, message: str, level: str = "INFO"):
        """Отправка уведомлений"""
        notifications = self.config.get('notifications', {})
        if not notifications.get('enabled', False):
            return

        try:
            # Webhook уведомление
            webhook_url = notifications.get('webhook_url')
            if webhook_url:
                payload = {
                    'text': f"[BingX Bot] {level}: {message}",
                    'timestamp': datetime.now().isoformat()
                }
                requests.post(webhook_url, json=payload, timeout=5)

            # Telegram уведомление
            bot_token = notifications.get('telegram_bot_token')
            chat_id = notifications.get('telegram_chat_id')
            if bot_token and chat_id:
                telegram_url = f"https://api.telegram.org/bot{bot_token}/sendMessage"
                payload = {
                    'chat_id': chat_id,
                    'text': f"🤖 BingX Bot\n{level}: {message}",
                    'parse_mode': 'HTML'
                }
                requests.post(telegram_url, json=payload, timeout=5)

        except Exception as e:
            self.logger.error(f"Ошибка отправки уведомления: {e}")
        
    def get_open_positions(self) -> List[Dict]:
        """Получение списка открытых позиций"""
        try:
            response = self.api.get_positions()
            if response.get('code') == 0:
                positions = response.get('data', [])
                # Фильтруем только открытые позиции
                open_positions = [pos for pos in positions if float(pos.get('positionAmt', 0)) != 0]
                return open_positions
            else:
                self.logger.error(f"Ошибка получения позиций: {response}")
                return []
        except Exception as e:
            self.logger.error(f"Исключение при получении позиций: {e}")
            return []
            
    def get_position_orders(self, symbol: str) -> List[Dict]:
        """Получение ордеров для конкретного символа"""
        try:
            response = self.api.get_open_orders(symbol=symbol)
            if response.get('code') == 0:
                return response.get('data', [])
            else:
                self.logger.error(f"Ошибка получения ордеров для {symbol}: {response}")
                return []
        except Exception as e:
            self.logger.error(f"Исключение при получении ордеров для {symbol}: {e}")
            return []
            
    def has_stop_loss(self, symbol: str) -> bool:
        """Проверка наличия stop loss ордера для позиции"""
        orders = self.get_position_orders(symbol)
        for order in orders:
            if order.get('type') == 'STOP_MARKET' or order.get('stopPrice'):
                return True
        return False
        
    def calculate_stop_loss_price(self, position: Dict) -> float:
        """Расчет цены stop loss"""
        entry_price = float(position.get('entryPrice', 0))
        side = position.get('positionSide', 'LONG')
        offset = self.config.get('stop_loss_offset', 0.005)
        
        if side == 'LONG':
            # Для лонга stop loss ниже цены входа
            return entry_price * (1 - offset)
        else:
            # Для шорта stop loss выше цены входа
            return entry_price * (1 + offset)
            
    def place_stop_loss(self, position: Dict) -> bool:
        """Размещение stop loss ордера"""
        try:
            symbol = position.get('symbol')
            position_amt = float(position.get('positionAmt', 0))
            side = position.get('positionSide', 'LONG')
            stop_price = self.calculate_stop_loss_price(position)
            
            # Определяем направление ордера (противоположное позиции)
            order_side = 'SELL' if side == 'LONG' else 'BUY'
            
            response = self.api.place_order(
                symbol=symbol,
                side=order_side,
                type='STOP_MARKET',
                quantity=abs(position_amt),
                stopPrice=stop_price,
                timeInForce='GTC'
            )
            
            if response.get('code') == 0:
                self.logger.info(f"Stop loss установлен для {symbol}: {stop_price}")
                return True
            else:
                self.logger.error(f"Ошибка установки stop loss для {symbol}: {response}")
                return False
                
        except Exception as e:
            self.logger.error(f"Исключение при установке stop loss для {symbol}: {e}")
            return False
            
    def calculate_profit_percentage(self, position: Dict) -> float:
        """Расчет процента прибыли без учета плеча"""
        try:
            entry_price = float(position.get('entryPrice', 0))
            mark_price = float(position.get('markPrice', 0))
            side = position.get('positionSide', 'LONG')
            
            if entry_price == 0 or mark_price == 0:
                return 0.0
                
            if side == 'LONG':
                profit_pct = (mark_price - entry_price) / entry_price
            else:
                profit_pct = (entry_price - mark_price) / entry_price
                
            return profit_pct
            
        except Exception as e:
            self.logger.error(f"Ошибка расчета прибыли: {e}")
            return 0.0

    def partial_close_position(self, position: Dict) -> bool:
        """Частичное закрытие позиции (50%)"""
        try:
            symbol = position.get('symbol')
            position_amt = float(position.get('positionAmt', 0))
            side = position.get('positionSide', 'LONG')

            # Рассчитываем количество для закрытия (50%)
            close_quantity = abs(position_amt) * self.config.get('partial_close_percent', 0.5)

            # Определяем направление ордера (противоположное позиции)
            order_side = 'SELL' if side == 'LONG' else 'BUY'

            response = self.api.place_order(
                symbol=symbol,
                side=order_side,
                type='MARKET',
                quantity=close_quantity,
                reduceOnly=True
            )

            if response.get('code') == 0:
                self.logger.info(f"Частично закрыта позиция {symbol}: {close_quantity}")
                return True
            else:
                self.logger.error(f"Ошибка частичного закрытия {symbol}: {response}")
                return False

        except Exception as e:
            self.logger.error(f"Исключение при частичном закрытии {symbol}: {e}")
            return False

    def move_stop_loss_to_breakeven(self, position: Dict) -> bool:
        """Перевод stop loss в безубыток"""
        try:
            symbol = position.get('symbol')
            entry_price = float(position.get('entryPrice', 0))

            # Сначала отменяем существующие stop loss ордера
            orders = self.get_position_orders(symbol)
            for order in orders:
                if order.get('type') == 'STOP_MARKET' or order.get('stopPrice'):
                    cancel_response = self.api.cancel_order(
                        symbol=symbol,
                        orderId=order.get('orderId')
                    )
                    if cancel_response.get('code') == 0:
                        self.logger.info(f"Отменен старый stop loss для {symbol}")

            # Устанавливаем новый stop loss в безубыток
            position_amt = float(position.get('positionAmt', 0))
            side = position.get('positionSide', 'LONG')
            order_side = 'SELL' if side == 'LONG' else 'BUY'

            # Оставшееся количество после частичного закрытия
            remaining_quantity = abs(position_amt) * (1 - self.config.get('partial_close_percent', 0.5))

            response = self.api.place_order(
                symbol=symbol,
                side=order_side,
                type='STOP_MARKET',
                quantity=remaining_quantity,
                stopPrice=entry_price,
                timeInForce='GTC'
            )

            if response.get('code') == 0:
                self.logger.info(f"Stop loss переведен в безубыток для {symbol}: {entry_price}")
                return True
            else:
                self.logger.error(f"Ошибка перевода stop loss в безубыток для {symbol}: {response}")
                return False

        except Exception as e:
            self.logger.error(f"Исключение при переводе stop loss в безубыток для {symbol}: {e}")
            return False

    def should_process_symbol(self, symbol: str) -> bool:
        """Проверка, нужно ли обрабатывать данный символ"""
        enabled_symbols = self.config.get('enabled_symbols', [])
        disabled_symbols = self.config.get('disabled_symbols', [])

        # Если указан список разрешенных символов, проверяем его
        if enabled_symbols and symbol not in enabled_symbols:
            return False

        # Проверяем список запрещенных символов
        if symbol in disabled_symbols:
            return False

        return True

    def process_position(self, position: Dict):
        """Обработка одной позиции"""
        symbol = position.get('symbol')

        if not self.should_process_symbol(symbol):
            return

        # Проверяем минимальный размер позиции
        notional = float(position.get('notional', 0))
        min_size = self.config.get('min_position_size', 10)
        if abs(notional) < min_size:
            return

        # Проверяем максимальную стоимость позиции
        risk_config = self.config.get('risk_management', {})
        max_position_value = risk_config.get('max_position_value', 1000)
        if abs(notional) > max_position_value:
            self.logger.warning(f"Позиция {symbol} превышает максимальную стоимость: {abs(notional)} > {max_position_value}")
            self.send_notification(f"Позиция {symbol} превышает лимит: {abs(notional):.2f} USDT", "WARNING")
            return

        self.logger.info(f"Обрабатываем позицию: {symbol} (размер: {abs(notional):.2f} USDT)")

        try:
            # 1. Проверяем наличие stop loss
            if not self.has_stop_loss(symbol):
                self.logger.warning(f"Отсутствует stop loss для {symbol}, добавляем...")
                if self.config.get('dry_run', False):
                    self.logger.info(f"[DRY RUN] Установка stop loss для {symbol}")
                else:
                    if self.place_stop_loss(position):
                        self.daily_trades += 1
                        self.send_notification(f"Установлен stop loss для {symbol}", "INFO")

            # 2. Проверяем прибыль
            profit_pct = self.calculate_profit_percentage(position)
            profit_threshold = self.config.get('profit_threshold', PROFIT_THRESHOLD)

            if profit_pct >= profit_threshold:
                self.logger.info(f"Позиция {symbol} в прибыли {profit_pct:.2%}, выполняем частичное закрытие")

                if self.config.get('dry_run', False):
                    self.logger.info(f"[DRY RUN] Частичное закрытие {symbol} и перевод в безубыток")
                else:
                    # Частично закрываем позицию
                    if self.partial_close_position(position):
                        self.daily_trades += 1
                        self.send_notification(f"Частично закрыта позиция {symbol} с прибылью {profit_pct:.2%}", "SUCCESS")

                        # Переводим stop loss в безубыток
                        time.sleep(1)  # Небольшая задержка
                        if self.move_stop_loss_to_breakeven(position):
                            self.send_notification(f"Stop loss переведен в безубыток для {symbol}", "INFO")

        except Exception as e:
            self.error_count += 1
            self.logger.error(f"Ошибка обработки позиции {symbol}: {e}")
            self.send_notification(f"Ошибка обработки позиции {symbol}: {e}", "ERROR")

    def run_check_cycle(self):
        """Один цикл проверки всех позиций"""
        # Сбрасываем дневные счетчики если нужно
        self.reset_daily_counters()

        # Проверяем аварийную остановку
        if self.check_emergency_stop():
            self.logger.warning("Пропускаем цикл из-за аварийной остановки")
            return

        self.logger.info("Начинаем проверку позиций...")

        try:
            positions = self.get_open_positions()
            self.logger.info(f"Найдено открытых позиций: {len(positions)}")

            if len(positions) == 0:
                self.logger.info("Открытых позиций нет")
                return

            for position in positions:
                # Проверяем аварийную остановку перед каждой позицией
                if self.check_emergency_stop():
                    self.logger.warning("Прерываем обработку позиций из-за аварийной остановки")
                    break

                self.process_position(position)
                time.sleep(0.5)  # Небольшая задержка между обработкой позиций

        except Exception as e:
            self.error_count += 1
            self.logger.error(f"Ошибка в цикле проверки: {e}")
            self.send_notification(f"Ошибка в цикле проверки: {e}", "ERROR")

        self.logger.info(f"Цикл проверки завершен (сделок сегодня: {self.daily_trades}, ошибок: {self.error_count})")

    def run(self):
        """Основной цикл работы"""
        self.logger.info("Запуск BingX Position Manager")
        self.logger.info(f"Интервал проверки: {self.config.get('check_interval', CHECK_INTERVAL)} секунд")
        self.logger.info(f"Порог прибыли: {self.config.get('profit_threshold', PROFIT_THRESHOLD):.1%}")

        try:
            while True:
                self.run_check_cycle()

                # Ждем до следующей проверки
                interval = self.config.get('check_interval', CHECK_INTERVAL)
                self.logger.info(f"Ожидание {interval} секунд до следующей проверки...")
                time.sleep(interval)

        except KeyboardInterrupt:
            self.logger.info("Получен сигнал остановки, завершаем работу...")
        except Exception as e:
            self.logger.error(f"Критическая ошибка: {e}")

def main():
    """Главная функция"""
    print("BingX Position Manager v1.0")
    print("Автоматическое управление позициями на BingX")
    print("-" * 50)

    try:
        manager = BingXPositionManager()
        manager.run()
    except Exception as e:
        print(f"Ошибка запуска: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
