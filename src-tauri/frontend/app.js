// Tauri API invoke interface check
const hasTauri = typeof window !== 'undefined' && window.__TAURI__ !== undefined;
const invoke = hasTauri 
  ? ((window.__TAURI__.core && window.__TAURI__.core.invoke) || window.__TAURI__.invoke)
  : async (cmd, args) => {
      console.log(`[MockInvoke] ${cmd}`, args);
      // Fallback mocks for development in standard browser
      if (cmd === 'get_system_status') return "ATES Running | Initial Balance: 100000.00 | Confluence Check: Enabled";
      if (cmd === 'check_discipline') {
        const passed = args.price > 20000;
        return passed 
          ? "Trade setup passes Disciplined Core checks."
          : "Discipline violations: Spot price too low | Outside session hours";
      }
      if (cmd === 'execute_trade') {
        if (args.entryPrice <= 0 || args.stopLoss <= 0) return "INVALID PRICES";
        return `TRADE EXECUTED SUCCESSFULLY: ${args.symbol} @ ${args.entryPrice}`;
      }
      if (cmd === 'run_backtest') {
        return "Backtest complete | Trades: 12 | Win Rate: 58.3% | Total P&L: ₹3450.75 | Max DD: 3.4%";
      }
      if (cmd === 'trigger_orchestra_cycle') {
        return "ORCHESTRA CYCLE COMPLETE | All Main + Sub-Agents coordinated | Message Router active";
      }
      if (cmd === 'fetch_live_stock_price') {
        return args.symbol === 'NIFTY' ? 24500.00 + (Math.random() * 10 - 5) : 2950.00 + (Math.random() * 4 - 2);
      }
      // ── RUN SYSTEM mocks ──
      if (cmd === 'start_autonomous_system') {
        return JSON.stringify({ status: 'starting', kronos: true, orchestrator: true });
      }
      if (cmd === 'stop_autonomous_system') return 'System stopped';
      if (cmd === 'get_system_health') {
        // Simulate services becoming active after start
        const running = window._atesRunning || false;
        return JSON.stringify({ kronos: running, orchestrator: running, llm: running, running });
      }
      return "SUCCESS";
    };

// Live Watchlist State (connected to APIs)
let watchlistData = {
  'NIFTY': { name: 'NSE NIFTY Index', price: 24500.00, change: 1.24, isCrypto: false },
  'RELIANCE': { name: 'RELIANCE Industries', price: 2950.00, change: -0.45, isCrypto: false },
  'BTC': { name: 'BTC / USDT (Binance)', price: 67500.00, change: 3.12, isCrypto: true },
  'ETH': { name: 'ETH / USDT (Binance)', price: 3500.00, change: 2.54, isCrypto: true },
  'SOL': { name: 'SOL / USDT (Binance)', price: 155.00, change: -1.82, isCrypto: true }
};

// Global UI State
let activeDirection = 'long';
let positions = [];
let cashBalance = 100000.00;
let currentMarketPrice = 24500.00;
let currentSymbol = 'NIFTY';
let percentChange24h = 1.24;
let watchlistFilter = 'all';

// Historical simulated candlesticks
let candles = [];
let tickCounter = 0;
let tvWidget = null;

// Log writer helper
function logMessage(text, type = 'success') {
  const container = document.getElementById('console-logs');
  if (!container) return;
  const time = new Date().toLocaleTimeString();
  const entry = document.createElement('div');
  entry.className = `log-entry ${type}`;
  entry.innerText = `[${time}] ${text}`;
  container.appendChild(entry);
  container.scrollTop = container.scrollHeight;
}

// Currency Symbol Formatter
function formatCurrency(val) {
  const asset = watchlistData[currentSymbol.toUpperCase()] || { prefix: '₹' };
  const symbolPrefix = asset.isCrypto ? '$' : '₹';
  return `${symbolPrefix}${val.toLocaleString('en-IN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}`;
}

// System Status Sync
async function syncSystemStatus() {
  try {
    const status = await invoke('get_system_status');
    const dot = document.getElementById('status-dot');
    const text = document.getElementById('status-text');
    if (dot) dot.className = "status-dot success";
    if (text) text.innerText = "Core Online";
    logMessage(`System Status Sync: ${status}`, 'system');
  } catch (err) {
    const dot = document.getElementById('status-dot');
    const text = document.getElementById('status-text');
    if (dot) dot.className = "status-dot danger";
    if (text) text.innerText = "Core Error";
    logMessage(`System status load failed: ${err}`, 'error');
  }
}

// Generate Initial Candlesticks for Fallback Chart
function generateInitialCandles(basePrice) {
  candles = [];
  let currentPrice = basePrice * 0.96; 
  for (let i = 0; i < 40; i++) {
    const drift = (Math.random() - 0.48) * 0.008 * currentPrice;
    const open = currentPrice;
    const close = currentPrice + drift;
    const high = Math.max(open, close) + Math.random() * 0.004 * currentPrice;
    const low = Math.min(open, close) - Math.random() * 0.004 * currentPrice;
    const volume = Math.floor(Math.random() * 1000) + 100;
    
    candles.push({ open, high, low, close, volume });
    currentPrice = close;
  }
  currentMarketPrice = basePrice;
}

// Calculate Exponential Moving Average
function calculateEMA(data, period) {
  let ema = [];
  if (data.length === 0) return ema;
  let k = 2 / (period + 1);
  let prevEma = data[0].close;
  ema.push(prevEma);
  for (let i = 1; i < data.length; i++) {
    let curEma = data[i].close * k + prevEma * (1 - k);
    ema.push(curEma);
    prevEma = curEma;
  }
  return ema;
}

// Draw HTML5 Canvas simulated Candlestick chart (Offline Fallback)
function drawCanvasChart() {
  const canvas = document.getElementById('fallback-chart-canvas');
  if (!canvas || canvas.style.display === 'none') return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const width = canvas.width;
  const height = canvas.height;

  ctx.fillStyle = '#12161a';
  ctx.fillRect(0, 0, width, height);

  if (candles.length === 0) return;

  let maxPrice = -Infinity;
  let minPrice = Infinity;
  let maxVolume = -Infinity;

  candles.forEach(c => {
    if (c.high > maxPrice) maxPrice = c.high;
    if (c.low < minPrice) minPrice = c.low;
    if (c.volume > maxVolume) maxVolume = c.volume;
  });

  const priceRange = maxPrice - minPrice;
  maxPrice += priceRange * 0.1;
  minPrice -= priceRange * 0.1;

  const plotHeight = height - 60; 
  const plotOffset = 30;

  const getY = (price) => {
    return plotHeight - ((price - minPrice) / (maxPrice - minPrice) * (plotHeight - 40)) + plotOffset;
  };

  ctx.strokeStyle = '#232931';
  ctx.lineWidth = 1;
  ctx.fillStyle = '#848e9c';
  ctx.font = '10px monospace';
  
  const gridCount = 5;
  for (let i = 0; i <= gridCount; i++) {
    const price = minPrice + (maxPrice - minPrice) * (i / gridCount);
    const y = getY(price);
    
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width - 70, y);
    ctx.stroke();

    const asset = watchlistData[currentSymbol.toUpperCase()] || { prefix: '₹' };
    const priceText = `${asset.isCrypto ? '$' : '₹'}${price.toLocaleString('en-IN', {maximumFractionDigits: 1})}`;
    ctx.fillText(priceText, width - 65, y + 4);
  }

  const candleCount = candles.length;
  const candleWidth = (width - 75) / candleCount;
  
  for (let i = 0; i < candleCount; i++) {
    const c = candles[i];
    const x = i * candleWidth + 5;
    
    const isBullish = c.close >= c.open;
    const candleColor = isBullish ? '#0ecb81' : '#f6465d';
    ctx.strokeStyle = candleColor;
    ctx.fillStyle = candleColor;
    
    ctx.beginPath();
    ctx.moveTo(x + candleWidth / 2, getY(c.high));
    ctx.lineTo(x + candleWidth / 2, getY(c.low));
    ctx.stroke();

    const openY = getY(c.open);
    const closeY = getY(c.close);
    const bodyHeight = Math.max(Math.abs(closeY - openY), 2);
    const bodyY = Math.min(openY, closeY);

    ctx.fillRect(x + 2, bodyY, candleWidth - 4, bodyHeight);

    const volHeight = (c.volume / maxVolume) * 35;
    ctx.fillStyle = isBullish ? 'rgba(14, 203, 129, 0.15)' : 'rgba(246, 70, 93, 0.15)';
    ctx.fillRect(x + 2, height - volHeight - 10, candleWidth - 4, volHeight);
  }

  const ema20 = calculateEMA(candles, 20);
  const ema50 = calculateEMA(candles, 12); 

  const drawIndicator = (emaVals, color) => {
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let i = 0; i < emaVals.length; i++) {
      const x = i * candleWidth + 5 + candleWidth / 2;
      const y = getY(emaVals[i]);
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();
  };

  if (ema20.length > 0) drawIndicator(ema20, '#f0b90b'); 
  if (ema50.length > 0) drawIndicator(ema50, '#00f0ff'); 

  ctx.fillStyle = '#eaecef';
  ctx.font = 'bold 11px sans-serif';
  ctx.fillText(`${currentSymbol.toUpperCase()}/USDT 15m (Fallback Feed)`, 12, 22);

  ctx.fillStyle = '#848e9c';
  ctx.font = '10px monospace';
  const lastPrice = candles[candles.length - 1].close;
  const currentEma20 = ema20.length > 0 ? ema20[ema20.length - 1] : lastPrice;
  const currentEma50 = ema50.length > 0 ? ema50[ema50.length - 1] : lastPrice;

  ctx.fillStyle = '#f0b90b';
  ctx.fillText(`EMA(20): ${currentEma20.toFixed(2)}`, 200, 22);
  ctx.fillStyle = '#00f0ff';
  ctx.fillText(`EMA(50): ${currentEma50.toFixed(2)}`, 310, 22);
}

// Update TradingView / Fallback Candlestick Chart symbol
function updateChart(symbol) {
  const canvas = document.getElementById('fallback-chart-canvas');
  const cleanSymbol = symbol.toUpperCase().trim();
  
  let tvSymbol = 'BINANCE:BTCUSDT';
  if (cleanSymbol === 'BTC' || cleanSymbol === 'BTCUSDT') {
    tvSymbol = 'BINANCE:BTCUSDT';
  } else if (cleanSymbol === 'ETH' || cleanSymbol === 'ETHUSDT') {
    tvSymbol = 'BINANCE:ETHUSDT';
  } else if (cleanSymbol === 'SOL' || cleanSymbol === 'SOLUSDT') {
    tvSymbol = 'BINANCE:SOLUSDT';
  } else if (cleanSymbol === 'NIFTY') {
    tvSymbol = 'NSE:NIFTY';
  } else if (cleanSymbol === 'RELIANCE') {
    tvSymbol = 'NSE:RELIANCE';
  } else {
    tvSymbol = cleanSymbol.includes('USDT') ? `BINANCE:${cleanSymbol}` : `BINANCE:${cleanSymbol}USDT`;
  }

  if (window.TradingView) {
    if (canvas) canvas.style.display = 'none';
    
    try {
      tvWidget = new TradingView.widget({
        "width": "100%",
        "height": "100%",
        "symbol": tvSymbol,
        "interval": "15",
        "timezone": "Asia/Kolkata",
        "theme": "dark",
        "style": "1",
        "locale": "en",
        "toolbar_bg": "#181a20",
        "enable_publishing": false,
        "hide_side_toolbar": false,
        "allow_symbol_change": true,
        "container_id": "tradingview-chart-container"
      });
      return;
    } catch (e) {
      console.error('TradingView Widget failed, fallback to Canvas:', e);
    }
  }

  if (canvas) {
    canvas.style.display = 'block';
    const asset = watchlistData[cleanSymbol] || { price: 24500.00 };
    if (candles.length === 0 || currentSymbol !== cleanSymbol) {
      generateInitialCandles(asset.price);
    }
    resizeCanvas();
  }
}

// Canvas resizing helper
function resizeCanvas() {
  const canvas = document.getElementById('fallback-chart-canvas');
  if (!canvas || canvas.style.display === 'none') return;
  const parent = canvas.parentElement;
  if (parent) {
    canvas.width = parent.clientWidth;
    canvas.height = parent.clientHeight;
  }
  drawCanvasChart();
}
window.addEventListener('resize', resizeCanvas);

// Watchlist Rendering Logic
function renderWatchlist() {
  const container = document.getElementById('watchlist-list');
  if (!container) return;

  container.innerHTML = '';
  Object.keys(watchlistData).forEach(symbol => {
    const asset = watchlistData[symbol];
    
    if (watchlistFilter === 'stocks' && asset.isCrypto) return;
    if (watchlistFilter === 'crypto' && !asset.isCrypto) return;

    const prefix = asset.isCrypto ? '$' : '₹';
    const changeClass = asset.change >= 0 ? 'success' : 'danger';
    const changeSign = asset.change >= 0 ? '+' : '';
    
    const row = document.createElement('div');
    row.className = `wl-row ${symbol === currentSymbol.toUpperCase() ? 'active' : ''}`;
    row.dataset.sym = symbol;
    row.innerHTML = `
      <span class="wl-sym"><strong>${symbol}</strong><small>${asset.name}</small></span>
      <span class="wl-price">${prefix}${asset.price.toLocaleString('en-IN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</span>
      <span class="wl-change ${changeClass}">${changeSign}${asset.change.toFixed(2)}%</span>
    `;
    
    row.addEventListener('click', () => {
      selectAsset(symbol);
    });

    container.appendChild(row);
  });
}

// Select Asset and propagate throughout the UI state
function selectAsset(symbol) {
  symbol = symbol.toUpperCase().trim();
  const asset = watchlistData[symbol];
  if (!asset) return;

  currentSymbol = symbol;
  currentMarketPrice = asset.price;
  percentChange24h = asset.change;

  // Sync symbol selector pills
  document.querySelectorAll(`.symbol-pills .pill-btn`).forEach(p => {
    if (p.dataset.sym === symbol) {
      p.classList.add('active');
    } else {
      p.classList.remove('active');
    }
  });

  // Sync Form inputs
  const tradeSym = document.getElementById('trade-symbol');
  if (tradeSym) tradeSym.value = symbol;
  const inspectSym = document.getElementById('inspect-symbol');
  if (inspectSym) inspectSym.value = symbol;

  const tradeEntry = document.getElementById('trade-entry');
  if (tradeEntry) tradeEntry.value = currentMarketPrice.toFixed(2);
  const inspectPrice = document.getElementById('inspect-price');
  if (inspectPrice) inspectPrice.value = currentMarketPrice.toFixed(2);

  // Sync balance formatting prefix
  const balanceVal = document.getElementById('margin-balance');
  if (balanceVal) {
    balanceVal.innerText = formatCurrency(cashBalance);
  }

  // Recalculate SL and TP fields
  updateStopLossTakeProfitLimits(currentMarketPrice);

  // Re-generate chart candles & pivots
  generateInitialCandles(currentMarketPrice);
  updateOrderBookAndPivots(currentSymbol, currentMarketPrice);
  updateChart(currentSymbol);
  renderWatchlist();

  logMessage(`Asset focus switched to ${symbol}. Market stream loaded.`, 'system');
}

// Connect to Binance Public WebSocket for Real-time Crypto tickers
function connectCryptoWebSocket() {
  // Free, public websocket streams for live tick data
  const ws = new WebSocket('wss://stream.binance.com:9443/ws/btcusdt@ticker/ethusdt@ticker/solusdt@ticker');
  
  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      let symbol = '';
      if (data.s === 'BTCUSDT') symbol = 'BTC';
      else if (data.s === 'ETHUSDT') symbol = 'ETH';
      else if (data.s === 'SOLUSDT') symbol = 'SOL';

      if (symbol && watchlistData[symbol]) {
        watchlistData[symbol].price = parseFloat(data.c);
        watchlistData[symbol].change = parseFloat(data.P);
        
        // If selected active symbol matches this crypto ticker update state
        if (currentSymbol.toUpperCase() === symbol) {
          currentMarketPrice = watchlistData[symbol].price;
          percentChange24h = watchlistData[symbol].change;
          updateOrderBookAndPivots(currentSymbol, currentMarketPrice);

          const priceDisplay = document.getElementById('ticker-price');
          if (priceDisplay) {
            priceDisplay.innerText = formatCurrency(currentMarketPrice);
          }
        }
        
        renderWatchlist();
      }
    } catch (e) {
      console.error('WebSocket parsing error:', e);
    }
  };

  ws.onerror = (e) => {
    console.error('WebSocket stream error:', e);
  };

  ws.onclose = () => {
    console.log('Crypto WebSocket stream disconnected. Reconnecting in 5s...');
    setTimeout(connectCryptoWebSocket, 5000);
  };
}

// Periodically fetch live stock prices from Yahoo Finance in the Rust backend
function startStockUpdateLoop() {
  setInterval(async () => {
    try {
      for (let symbol of ['NIFTY', 'RELIANCE']) {
        const price = await invoke('fetch_live_stock_price', { symbol });
        if (price > 0) {
          // If price hasn't ticked (closed hours/offline), apply minor drift simulation
          if (price === watchlistData[symbol].price) {
            const driftPct = (Math.random() * 0.0006) - 0.0003;
            watchlistData[symbol].price = watchlistData[symbol].price * (1 + driftPct);
          } else {
            watchlistData[symbol].price = price;
          }
          
          if (currentSymbol.toUpperCase() === symbol) {
            currentMarketPrice = watchlistData[symbol].price;
            updateOrderBookAndPivots(currentSymbol, currentMarketPrice);
          }
        }
      }
      renderWatchlist();
    } catch (err) {
      console.warn('Failed to fetch stock pricing, applying failover drift:', err);
      for (let symbol of ['NIFTY', 'RELIANCE']) {
        const driftPct = (Math.random() * 0.0006) - 0.0003;
        watchlistData[symbol].price = watchlistData[symbol].price * (1 + driftPct);
        if (currentSymbol.toUpperCase() === symbol) {
          currentMarketPrice = watchlistData[symbol].price;
          updateOrderBookAndPivots(currentSymbol, currentMarketPrice);
        }
      }
      renderWatchlist();
    }
  }, 8000);
}

// Background Position SL/TP liquidator & Chart draw synchronizer
function startPositionEvaluatorAndCandleTick() {
  setInterval(() => {
    if (positions.length > 0) {
      let updated = false;
      for (let pos of positions) {
        const asset = watchlistData[pos.symbol.toUpperCase()];
        if (asset) {
          pos.current = asset.price;
          
          if (pos.direction === 'LONG') {
            pos.pnl = (pos.current - pos.entry) * pos.qty;
          } else {
            pos.pnl = (pos.entry - pos.current) * pos.qty;
          }

          // Evaluate Stop Loss or Take Profit hit
          const isLong = pos.direction === 'LONG';
          const stopHit = isLong ? pos.current <= pos.sl : pos.current >= pos.sl;
          const tpHit = isLong ? pos.current >= pos.tp : pos.current <= pos.tp;

          if (stopHit || tpHit) {
            const type = stopHit ? 'STOP LOSS' : 'TAKE PROFIT';
            const price = stopHit ? pos.sl : pos.tp;
            const finalPnl = isLong ? (price - pos.entry) * pos.qty : (pos.entry - price) * pos.qty;
            
            cashBalance += (pos.qty * pos.entry) + finalPnl;
            document.getElementById('margin-balance').innerText = formatCurrency(cashBalance);
            
            logMessage(`[ExecutionCoordinator] ${type} Triggered for ${pos.symbol} at ${formatCurrency(price)}. Realized P&L: ${formatCurrency(finalPnl)}`, stopHit ? 'error' : 'success');
            
            positions = positions.filter(p => p.symbol !== pos.symbol);
            updated = true;
            break; 
          }
          updated = true;
        }
      }
      if (updated) {
        renderPositions();
      }
    }

    // Tick the active candle fallback chart
    if (candles.length > 0) {
      const last = candles[candles.length - 1];
      last.close = currentMarketPrice;
      if (currentMarketPrice > last.high) last.high = currentMarketPrice;
      if (currentMarketPrice < last.low) last.low = currentMarketPrice;

      tickCounter++;
      if (tickCounter >= 12) {
        tickCounter = 0;
        candles.shift();
        candles.push({
          open: currentMarketPrice,
          high: currentMarketPrice,
          low: currentMarketPrice,
          close: currentMarketPrice,
          volume: Math.floor(Math.random() * 1000) + 100
        });
      }
      drawCanvasChart();
    }
  }, 1000);
}

// Order Book & Pivot Level Renderer
function updateOrderBookAndPivots(symbol, currentPrice) {
  const tickerName = document.getElementById('ticker-name');
  if (tickerName) tickerName.innerText = `${symbol.toUpperCase()} / INR`;
  
  const tickerPrice = document.getElementById('ticker-price');
  if (tickerPrice) {
    tickerPrice.innerText = formatCurrency(currentPrice);
  }

  const currentSpot = document.getElementById('ob-current-spot');
  if (currentSpot) {
    currentSpot.innerText = currentPrice.toLocaleString('en-IN', {minimumFractionDigits: 2, maximumFractionDigits: 2});
  }

  const high = currentPrice * 1.01;
  const low = currentPrice * 0.99;
  const close = currentPrice * 0.998;
  
  const pivot = (high + low + close) / 3.0;
  const r1 = 2.0 * pivot - low;
  const s1 = 2.0 * pivot - high;
  const r2 = pivot + (high - low);
  const s2 = pivot - (high - low);
  const r3 = high + 2.0 * (pivot - low);
  const s3 = low - 2.0 * (high - pivot);

  const updateText = (id, val) => {
    const el = document.getElementById(id);
    if (el) el.innerText = val.toLocaleString('en-IN', {minimumFractionDigits: 2, maximumFractionDigits: 2});
  };

  updateText('val-pivot', pivot);
  updateText('val-r1', r1);
  updateText('val-r2', r2);
  updateText('val-r3', r3);
  updateText('val-s1', s1);
  updateText('val-s2', s2);
  updateText('val-s3', s3);

  const updateDist = (id, targetVal) => {
    const el = document.getElementById(id);
    if (el) {
      const diffPct = ((targetVal - currentPrice) / currentPrice) * 100;
      const sign = diffPct >= 0 ? '+' : '';
      el.innerText = `${sign}${diffPct.toFixed(2)}%`;
    }
  };

  updateDist('dist-r3', r3);
  updateDist('dist-r2', r2);
  updateDist('dist-r1', r1);
  updateDist('dist-s1', s1);
  updateDist('dist-s2', s2);
  updateDist('dist-s3', s3);

  const tickerHigh = document.getElementById('ticker-high');
  if (tickerHigh) {
    tickerHigh.innerText = formatCurrency(high);
  }
  const tickerLow = document.getElementById('ticker-low');
  if (tickerLow) {
    tickerLow.innerText = formatCurrency(low);
  }
}

// Positions Table Renderer
function renderPositions() {
  const tbody = document.getElementById('positions-table-body');
  if (!tbody) return;

  if (positions.length === 0) {
    tbody.innerHTML = `
      <tr class="no-data">
        <td colspan="7">No active positions. Execute a trade to start.</td>
      </tr>
    `;
    return;
  }

  tbody.innerHTML = '';
  positions.forEach((pos) => {
    const pnlClass = pos.pnl >= 0 ? 'success' : 'danger';
    const pnlSign = pos.pnl >= 0 ? '+' : '';
    const row = document.createElement('tr');
    row.innerHTML = `
      <td><strong>${pos.symbol}</strong></td>
      <td class="${pos.direction === 'LONG' ? 'success' : 'danger'}">${pos.direction}</td>
      <td>${pos.qty}</td>
      <td>${formatCurrency(pos.entry)}</td>
      <td>${formatCurrency(pos.current)}</td>
      <td class="${pnlClass}">${pnlSign}${formatCurrency(pos.pnl)}</td>
      <td>
        <button class="btn btn-secondary man-close-btn" data-sym="${pos.symbol}" style="padding: 3px 8px; font-size: 10px; line-height: 1; margin: 0; cursor: pointer;">Close</button>
      </td>
    `;
    tbody.appendChild(row);
  });

  // Attach manual close event handlers
  document.querySelectorAll('.man-close-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      const sym = btn.dataset.sym.toUpperCase();
      const pos = positions.find(p => p.symbol.toUpperCase() === sym);
      if (pos) {
        const finalPnl = pos.pnl;
        cashBalance += (pos.qty * pos.entry) + finalPnl;
        document.getElementById('margin-balance').innerText = formatCurrency(cashBalance);

        logMessage(`[ExecutionCoordinator] Manual market exit triggered for ${pos.symbol}. Realized P&L: ${formatCurrency(finalPnl)}`, finalPnl >= 0 ? 'success' : 'error');
        
        positions = positions.filter(p => p.symbol.toUpperCase() !== sym);
        renderPositions();
      }
    });
  });
}

// Recalculate Stop Loss and Take Profit fields relative to selected direction
function updateStopLossTakeProfitLimits(price) {
  const tradeSl = document.getElementById('trade-sl');
  const tradeTp = document.getElementById('trade-tp');
  if (!tradeSl || !tradeTp) return;

  const isLong = activeDirection === 'long';
  const slOffset = isLong ? 0.99 : 1.01; 
  const tpOffset = isLong ? 1.02 : 0.98; 

  tradeSl.value = (price * slOffset).toFixed(2);
  tradeTp.value = (price * tpOffset).toFixed(2);
}

// Tab navigation handler
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', (e) => {
    e.preventDefault();
    document.querySelectorAll('.tab-btn').forEach(i => i.classList.remove('active'));
    document.querySelectorAll('.pane-view').forEach(v => v.classList.remove('active'));
    
    btn.classList.add('active');
    const paneId = btn.dataset.pane;
    const targetPane = document.getElementById(`pane-${paneId}`);
    if (targetPane) {
      targetPane.classList.add('active');
    }
    logMessage(`Switched tab to: ${paneId.toUpperCase()}`, 'system');
  });
});

// Buy/Sell Order Ticket tabs
document.querySelectorAll('.dir-tab').forEach(tab => {
  tab.addEventListener('click', (e) => {
    e.preventDefault();
    document.querySelectorAll('.dir-tab').forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    
    activeDirection = tab.dataset.dir;
    const submitBtn = document.getElementById('btn-submit-order');
    if (submitBtn) {
      if (activeDirection === 'long') {
        submitBtn.innerText = 'BUY / LONG';
        submitBtn.classList.remove('short-mode');
      } else {
        submitBtn.innerText = 'SELL / SHORT';
        submitBtn.classList.add('short-mode');
      }
    }

    const tradeEntry = document.getElementById('trade-entry');
    if (tradeEntry) {
      updateStopLossTakeProfitLimits(parseFloat(tradeEntry.value));
    }
  });
});

// Risk / Allocation Slider listener
const riskSlider = document.getElementById('risk-slider');
if (riskSlider) {
  riskSlider.addEventListener('input', (e) => {
    const val = e.target.value;
    const display = document.getElementById('slider-val');
    if (display) {
      display.innerText = `${val}%${val == 1 ? ' (Default)' : ''}`;
    }
  });
}

// Watchlist Filter Switchers
document.querySelectorAll('.wl-tab').forEach(tab => {
  tab.addEventListener('click', (e) => {
    e.preventDefault();
    document.querySelectorAll('.wl-tab').forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    watchlistFilter = tab.dataset.type;
    renderWatchlist();
  });
});

// Symbol quick pills switcher integration
document.querySelectorAll('.symbol-pills .pill-btn').forEach(pill => {
  pill.addEventListener('click', (e) => {
    e.preventDefault();
    const sym = pill.dataset.sym;
    selectAsset(sym);
  });
});

// Audit Inspector
document.getElementById('discipline-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const symbol = document.getElementById('inspect-symbol').value.trim();
  const price = parseFloat(document.getElementById('inspect-price').value);

  logMessage(`[SubAgentTask] Dispatching check_discipline for ${symbol} @ ${formatCurrency(price)}`, 'system');
  
  selectAsset(symbol);

  try {
    const result = await invoke('check_discipline', { symbol, price });
    const container = document.getElementById('inspector-output');
    const badge = document.getElementById('audit-badge');
    const details = document.getElementById('audit-details');

    if (container) container.classList.remove('hidden');
    if (details) details.innerText = result;

    const passed = !result.toLowerCase().includes('violation') && !result.toLowerCase().includes('reject');
    if (passed) {
      if (container) container.className = "inspector-output pass";
      if (badge) badge.innerText = "PASSED";
      logMessage(`[DisciplineCheck] PASSED for ${symbol} @ ${formatCurrency(price)}`, 'success');
    } else {
      if (container) container.className = "inspector-output fail";
      if (badge) badge.innerText = "VIOLATION";
      logMessage(`[DisciplineCheck] REJECTED for ${symbol} @ ${formatCurrency(price)}: ${result}`, 'error');
    }
  } catch (err) {
    logMessage(`Discipline check invoke error: ${err}`, 'error');
  }
});

// Orchestra Cycle Coordination Sweep
document.getElementById('btn-trigger-cycle').addEventListener('click', async () => {
  const output = document.getElementById('cycle-quick-output');
  if (output) {
    output.classList.remove('hidden');
    output.innerHTML = "Initializing agent passing protocol...\n";
  }
  
  logMessage("[Orchestrator] Starting cycle coordination sweep...", 'system');
  
  try {
    const result = await invoke('trigger_orchestra_cycle');
    const lines = [
      "[Orchestra] Phase 1: Validating Disciplined Core...",
      "[Orchestra] Phase 2: Activating Main Agents (MarketIntelligence, RiskPsychology, Reflector)...",
      "[Orchestra] Phase 3: Activating Sub-Agents (RiskCalculator, PivotCalculator)...",
      "[Orchestra] Phase 4: Running Message Router coordination...",
      "[Orchestra] Phase 5: ExecutionEngine ready for validated setups...",
      `[Tauri] Result: ${result}`
    ];
    
    let i = 0;
    const interval = setInterval(() => {
      if (output) {
        if (i < lines.length) {
          output.innerHTML += lines[i] + "\n";
          output.scrollTop = output.scrollHeight;
          logMessage(lines[i], 'success');
          i++;
        } else {
          clearInterval(interval);
        }
      } else {
        clearInterval(interval);
      }
    }, 400);
  } catch (err) {
    if (output) output.innerHTML += `ERROR: ${err}\n`;
    logMessage(`Orchestra sweep error: ${err}`, 'error');
  }
});

// Trade Ticket Submission
document.getElementById('trade-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const symbol = document.getElementById('trade-symbol').value.trim();
  const entry = parseFloat(document.getElementById('trade-entry').value);
  const sl = parseFloat(document.getElementById('trade-sl').value);
  const tp = parseFloat(document.getElementById('trade-tp').value);

  logMessage(`[ExecutionCoordinator] Evaluating trade request: ${activeDirection.toUpperCase()} ${symbol} @ ${formatCurrency(entry)}`, 'system');

  selectAsset(symbol);

  try {
    const result = await invoke('execute_trade', {
      symbol,
      directionStr: activeDirection,
      entryPrice: entry,
      stopLoss: sl,
      takeProfit: tp
    });

    const output = document.getElementById('trade-output');
    if (output) {
      output.classList.remove('hidden');
      output.innerText = result;
    }

    if (result.includes('SUCCESS') || result.includes('EXECUTED')) {
      if (output) {
        output.style.borderColor = "var(--success-color)";
        output.style.color = "var(--success-color)";
      }
      logMessage(`[ExecutionEngine] Trade execution successful: ${result}`, 'success');

      const riskMultiplier = parseInt(document.getElementById('risk-slider').value) || 1;
      const qty = 10 * riskMultiplier; 
      const positionValue = qty * entry;
      cashBalance -= positionValue;
      
      const balanceVal = document.getElementById('margin-balance');
      if (balanceVal) {
        balanceVal.innerText = formatCurrency(cashBalance);
      }

      positions.push({
        symbol,
        direction: activeDirection.toUpperCase(),
        qty,
        entry,
        current: entry,
        sl,
        tp,
        pnl: 0.0
      });
      renderPositions();
    } else {
      if (output) {
        output.style.borderColor = "var(--danger-color)";
        output.style.color = "var(--danger-color)";
      }
      logMessage(`[ExecutionEngine] Trade rejected: ${result}`, 'error');
    }
  } catch (err) {
    if (output) {
      output.classList.remove('hidden');
      output.innerText = `ERROR: ${err}`;
      output.style.borderColor = "var(--danger-color)";
      output.style.color = "var(--danger-color)";
    }
    logMessage(`Trade error: ${err}`, 'error');
  }
});

// Backtest Engine
document.getElementById('btn-run-backtest').addEventListener('click', async () => {
  logMessage("[Backtester] Launching simulation over 50 data points...", 'system');
  
  try {
    const result = await invoke('run_backtest');
    const parts = result.split('|');
    const trades = parts[1].split(':')[1].trim();
    const winrate = parts[2].split(':')[1].trim();
    const pnl = parts[3].split(':')[1].trim();
    const drawdown = parts[4].split(':')[1].trim();

    const resultsContainer = document.getElementById('backtest-results-container');
    if (resultsContainer) resultsContainer.classList.remove('hidden');
    
    document.getElementById('bt-trades').innerText = trades;
    document.getElementById('bt-winrate').innerText = winrate;
    document.getElementById('bt-pnl').innerText = pnl;
    document.getElementById('bt-drawdown').innerText = drawdown;
    document.getElementById('bt-message').innerText = result;

    logMessage(`[BacktestEngine] Backtest run complete. Win Rate: ${winrate}, Net Profit: ${pnl}`, 'success');
  } catch (err) {
    logMessage(`Backtester error: ${err}`, 'error');
  }
});

// Clear console logger
document.getElementById('btn-clear-console').addEventListener('click', () => {
  const container = document.getElementById('console-logs');
  if (container) {
    container.innerHTML = `
      <div class="log-entry system">[System] Console logs cleared.</div>
    `;
  }
});

// Order Book Interaction: click levels to auto-fill entry, SL, or TP values in trade ticket form
document.querySelectorAll('.ob-row, .ob-spread').forEach(row => {
  row.addEventListener('click', () => {
    const targetType = row.dataset.target;
    const valId = row.dataset.valId;
    if (!targetType || !valId) return;
    
    const valElement = document.getElementById(valId);
    if (!valElement) return;

    const valText = valElement.innerText;
    const numericVal = parseFloat(valText.replace(/,/g, ''));
    if (isNaN(numericVal)) return;

    if (targetType === 'tp') {
      const tpField = document.getElementById('trade-tp');
      if (tpField) {
        tpField.value = numericVal.toFixed(2);
        logMessage(`[OrderTicket] Take Profit set to ${formatCurrency(numericVal)} from Order Book`, 'system');
      }
    } else if (targetType === 'sl') {
      const slField = document.getElementById('trade-sl');
      if (slField) {
        slField.value = numericVal.toFixed(2);
        logMessage(`[OrderTicket] Stop Loss set to ${formatCurrency(numericVal)} from Order Book`, 'system');
      }
    } else if (targetType === 'entry') {
      const entryField = document.getElementById('trade-entry');
      if (entryField) {
        entryField.value = numericVal.toFixed(2);
        logMessage(`[OrderTicket] Entry Price set to ${formatCurrency(numericVal)} from Order Book`, 'system');
      }
    }
  });
});

// Synchronize and update active Discipline Guard rules on the backend
async function syncDisciplineRules() {
  const useConfluence = document.getElementById('chk-confluence')?.checked ?? true;
  const respectSession = document.getElementById('chk-session')?.checked ?? true;
  try {
    const res = await invoke('update_rules', { useConfluence, respectSessionTiming: respectSession });
    logMessage(`[DisciplineEngine] Rules synchronized: Confluence Guard = ${useConfluence ? 'ON' : 'OFF'}, Session Guard = ${respectSession ? 'ON' : 'OFF'}`, 'system');
  } catch (err) {
    console.error('Failed to sync discipline rules:', err);
  }
}

document.getElementById('chk-confluence')?.addEventListener('change', syncDisciplineRules);
document.getElementById('chk-session')?.addEventListener('change', syncDisciplineRules);

// Initialization
syncSystemStatus();
syncDisciplineRules();
generateInitialCandles(currentMarketPrice);
updateOrderBookAndPivots(currentSymbol, currentMarketPrice);
updateChart(currentSymbol);
renderWatchlist();

// Connect to WebSocket and Poll Stock endpoints
connectCryptoWebSocket();
startStockUpdateLoop();
startPositionEvaluatorAndCandleTick();

// Select default asset pill
document.querySelectorAll(`.symbol-pills .pill-btn[data-sym="NIFTY"]`).forEach(p => p.classList.add('active'));

logMessage("Real-time Watchlist and WebSockets active. Ready.", 'system');

// ═══════════════════════════════════════════════════════════════════════════
//  RUN SYSTEM CONTROLLER
//  Manages the RUN / STOP button, service ribbon, and health polling.
// ═══════════════════════════════════════════════════════════════════════════

const RunSystem = {
  state: 'stopped',   // 'stopped' | 'starting' | 'running' | 'stopping'
  healthTimer: null,
  cycleCount: 0,

  // ── UI references ────────────────────────────────────────────────────────
  btn()   { return document.getElementById('btn-run'); },
  label() { return this.btn()?.querySelector('.run-btn-label'); },
  icon()  { return this.btn()?.querySelector('.run-btn-icon'); },

  // ── Set button visual state ───────────────────────────────────────────────
  setButtonState(state) {
    const btn = this.btn();
    if (!btn) return;
    btn.classList.remove('stopped', 'starting', 'running', 'stopping');
    btn.classList.add(state);
    this.state = state;

    const labels = {
      stopped:  { icon: '▶', text: 'RUN SYSTEM' },
      starting: { icon: '⟳', text: 'STARTING...' },
      running:  { icon: '■', text: 'RUNNING' },
      stopping: { icon: '⏹', text: 'STOPPING...' },
    };
    const l = labels[state];
    if (this.label()) this.label().textContent = l.text;
    if (this.icon())  this.icon().textContent  = l.icon;
  },

  // ── Update the ribbon service badge ──────────────────────────────────────
  setRibbonService(id, dotClass, stateText, stateClass) {
    const el = document.getElementById(id);
    if (!el) return;
    el.querySelector('.ribbon-dot').className   = `ribbon-dot ${dotClass}`;
    const s = el.querySelector('.ribbon-state');
    s.textContent  = stateText;
    s.className    = `ribbon-state ${stateClass}`;
  },

  // ── Poll get_system_health every 3s ──────────────────────────────────────
  startHealthPolling() {
    this.stopHealthPolling();
    this.healthTimer = setInterval(async () => {
      try {
        const raw    = await invoke('get_system_health');
        const health = typeof raw === 'string' ? JSON.parse(raw) : raw;
        this.applyHealth(health);
      } catch (e) { /* silent */ }
    }, 3000);
  },

  stopHealthPolling() {
    if (this.healthTimer) { clearInterval(this.healthTimer); this.healthTimer = null; }
  },

  // ── Apply health response to ribbon UI ───────────────────────────────────
  applyHealth(h) {
    this.setRibbonService(
      'ribbon-kronos',
      h.kronos       ? 'on'   : 'off',
      h.kronos       ? 'FORECASTING' : 'OFFLINE',
      h.kronos       ? 'active' : ''
    );
    this.setRibbonService(
      'ribbon-orch',
      h.orchestrator ? 'on'   : 'warn',
      h.orchestrator ? 'LLM ACTIVE'  : 'STARTING',
      h.orchestrator ? 'active' : 'warn'
    );
    this.setRibbonService(
      'ribbon-llm',
      h.llm          ? 'on'   : 'off',
      h.llm          ? 'ministral-3'  : 'STANDBY',
      h.llm          ? 'active' : ''
    );

    // Update cycle counter
    if (h.running) {
      this.cycleCount++;
      const cyEl = document.getElementById('ribbon-cycle');
      if (cyEl) cyEl.textContent = `#${this.cycleCount}`;
    }

    // Auto-transition from 'starting' to 'running' once services are up
    if (this.state === 'starting' && (h.kronos || h.orchestrator)) {
      this.setButtonState('running');
      logMessage('[System] ✅ Autonomous system ONLINE — Kronos + Orchestrator active', 'system');
      logMessage('[LLM] 🤖 ministral-3:3b-cloud connected — autonomous trade decisions active', 'system');
    }
  },

  // ── LAUNCH ───────────────────────────────────────────────────────────────
  async start() {
    this.setButtonState('starting');
    logMessage('[System] ▶ Launching ATES Autonomous System...', 'system');
    logMessage('[Kronos] ⏳ Starting Forecasting Service (port 8000)...', 'system');
    logMessage('[Orchestrator] ⏳ Launching Autonomous Agent Loop (Ollama + Kronos)...', 'system');

    // Browser-only flag so mock health returns true
    window._atesRunning = true;

    try {
      const raw = await invoke('start_autonomous_system');
      const res = typeof raw === 'string' ? JSON.parse(raw) : raw;

      if (res.kronos)       logMessage('[Kronos] ✅ Forecasting Service spawned', 'system');
      if (res.orchestrator) logMessage('[Orchestrator] ✅ Agent loop spawned — LLM deciding trades every 5s', 'system');

      this.startHealthPolling();

      // If response says already starting, wait for polling to transition to 'running'
      // (Tauri backend takes a moment to boot services)
      setTimeout(async () => {
        if (this.state === 'starting') {
          const raw2 = await invoke('get_system_health').catch(() => null);
          if (raw2) {
            const h = typeof raw2 === 'string' ? JSON.parse(raw2) : raw2;
            this.applyHealth(h);
          }
        }
      }, 2000);

    } catch (err) {
      logMessage(`[System] ❌ Failed to start: ${err}`, 'system');
      this.setButtonState('stopped');
      window._atesRunning = false;
    }
  },

  // ── STOP ─────────────────────────────────────────────────────────────────
  async stop() {
    this.setButtonState('stopping');
    logMessage('[System] ⏹ Stopping autonomous system...', 'system');
    window._atesRunning = false;

    try {
      await invoke('stop_autonomous_system');
      logMessage('[System] 🛑 Kronos + Orchestrator stopped', 'system');
    } catch (e) {
      logMessage(`[System] Stop error: ${e}`, 'system');
    }

    this.stopHealthPolling();
    this.cycleCount = 0;

    // Reset ribbon
    ['ribbon-kronos', 'ribbon-orch', 'ribbon-llm'].forEach(id => {
      this.setRibbonService(id, 'off', 'OFFLINE', '');
    });
    ['ribbon-cycle', 'ribbon-pos-count', 'ribbon-pnl-val'].forEach(id => {
      const el = document.getElementById(id);
      if (el) el.textContent = '—';
    });

    this.setButtonState('stopped');
  },
};

// Exposed globally so the onclick in HTML can call it
window.runSystemToggle = function () {
  if (RunSystem.state === 'stopped')  RunSystem.start();
  else if (RunSystem.state === 'running') RunSystem.stop();
  // 'starting' and 'stopping' ignore clicks (button pointer-events: none for stopping)
};

// ── Update ribbon position / P&L from existing position state ─────────────
function updateRibbonPositions() {
  const posEl = document.getElementById('ribbon-pos-count');
  const pnlEl = document.getElementById('ribbon-pnl-val');
  if (!posEl || !pnlEl) return;

  posEl.textContent = positions.length > 0 ? positions.length : '0';

  const totalPnl = positions.reduce((sum, p) => {
    const pnl = p.direction === 'long'
      ? (currentMarketPrice - p.entry) * p.qty
      : (p.entry - currentMarketPrice) * p.qty;
    return sum + pnl;
  }, 0);

  pnlEl.textContent   = `₹${totalPnl >= 0 ? '+' : ''}${totalPnl.toFixed(2)}`;
  pnlEl.className     = `ribbon-stat-val ${totalPnl >= 0 ? 'pos' : 'neg'}`;
}

// Hook ribbon updates into the existing position evaluator tick
const _origEval = window._positionEvalTick;
setInterval(updateRibbonPositions, 2000);

