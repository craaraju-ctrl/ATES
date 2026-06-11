import pandas as pd
import numpy as np

class KronosTokenizer:
    @classmethod
    def from_pretrained(cls, path):
        print(f"[MockTokenizer] Initialized from {path}")
        return cls()

class Kronos:
    @classmethod
    def from_pretrained(cls, path):
        print(f"[MockModel] Initialized from {path}")
        return cls()

class KronosPredictor:
    def __init__(self, model, tokenizer):
        self.model = model
        self.tokenizer = tokenizer

    def predict(self, df, x_timestamp, y_timestamp, pred_len, temperature=0.8, top_p=0.9, sample_count=1):
        print(f"[MockPredictor] Generating forecast for {pred_len} periods")
        # Get the last close price and other values
        last_close = df['close'].iloc[-1]
        last_open = df['open'].iloc[-1]
        last_high = df['high'].iloc[-1]
        last_low = df['low'].iloc[-1]
        last_volume = df['volume'].iloc[-1]

        # Generate mock forecast using a simple drift/random walk
        # Let's add a small upward drift (e.g. 0.05% per candle) to make it look active
        drift = 0.0005
        preds = []
        current_close = last_close
        for i in range(pred_len):
            # simulate a candle
            change = current_close * (drift + np.random.normal(0, 0.001))
            new_close = current_close + change
            new_open = current_close
            new_high = max(new_open, new_close) + abs(np.random.normal(0, current_close * 0.0005))
            new_low = min(new_open, new_close) - abs(np.random.normal(0, current_close * 0.0005))
            new_volume = last_volume * (1.0 + np.random.normal(0, 0.1))
            preds.append({
                'open': new_open,
                'high': new_high,
                'low': new_low,
                'close': new_close,
                'volume': new_volume
            })
            current_close = new_close

        pred_df = pd.DataFrame(preds, index=y_timestamp)
        return pred_df
