use anyhow::anyhow;
use t_invest_sdk::api::{Candle, SubscribeCandlesResponse, SubscriptionStatus};
use t_invest_sdk::api::{
    MarketDataResponse,
    market_data_response::Payload};
use std::env;
use t_invest_sdk::api::{
    get_candles_request::CandleSource, market_data_request, CandleInstrument,
    FindInstrumentRequest, InstrumentType, MarketDataRequest, SubscribeCandlesRequest,
    SubscriptionAction, SubscriptionInterval,
};
use t_invest_sdk::TInvestSdk;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let token = env::var("API_TOKEN")?;
    let sdk = TInvestSdk::new_sandbox(&token).await?;

    let find_instrument_response = sdk
        .instruments()
        .find_instrument(FindInstrumentRequest {
            query: "Т-Технологии".to_string(),
            instrument_kind: Some(InstrumentType::Share as i32),
            api_trade_available_flag: Some(true),
        })
        .await?
        .into_inner();

    let instrument = find_instrument_response
        .instruments
        .first()
        .ok_or(anyhow!("Can't find instrument"))?;

    println!("Instrument: {:?}", instrument);

    let (tx, rx) = flume::unbounded();
    let request = MarketDataRequest {
        payload: Some(market_data_request::Payload::SubscribeCandlesRequest(
            SubscribeCandlesRequest {
                subscription_action: SubscriptionAction::Subscribe as i32,
                instruments: vec![CandleInstrument {
                    figi: "".to_string(),
                    interval: SubscriptionInterval::OneMinute as i32,
                    instrument_id: instrument.uid.clone(),
                }],
                waiting_close: false,
                candle_source_type: Some(CandleSource::Unspecified as i32),
            },
        )),
    };
    tx.send(request)?;

    let response = sdk
        .market_data_stream()
        .market_data_stream(rx.into_stream())
        .await?;

    let mut streaming = response.into_inner();

    const rsi_n: u32 = 14;
    
    // Хранение данных: вам потребуется буфер значений close (длиной n+1, чтобы иметь предыдущее закрытие) и переменные текущих avg_gain, avg_loss.
    // Расчёт индикатора начинается только после накопления первых n свечей; до этого момента сигналы не генерируются.
    loop {
        if let Some(next_message) = streaming.message().await? {
            println!("Candle: {:?}", next_message);
            if let Some(payload) = next_message.payload  {
                handle_message(payload);   
            }
        }
    }
}

fn handle_message(next_message_payload: Payload) {
    if let Payload::Candle(candle_response) = next_message_payload {
        handle_candle(candle_response)
    }
}

fn handle_candle(candle_response: Candle) {

    let mut open = candle_response.open;
    let mut close = candle_response.close;

    let mut open = candle_response.open;
    let mut close = candle_response.close;

    // Период n – гиперпараметр (классически 14). На каждом закрытии свечи (цена close) вычисляем прирост или падение относительно предыдущего закрытия.
    gain_i = max(close_i - close_{i-1}, 0)
    loss_i = max(close_{i-1} - close_i, 0)

    // Первое сглаженное среднее (за первые n свечей)
    avg_gain = (gain_2 + gain_3 + ... + gain_n) / n
    avg_loss = (loss_2 + loss_3 + ... + loss_n) / n

    // Последующие значения (для свечи n+1 и далее) обновляются по методу Уайлдера (экспоненциальное сглаживание):
    avg_gain = (prev_avg_gain * (n - 1) + gain_current) / n
    avg_loss = (prev_avg_loss * (n - 1) + loss_current) / n

    RS = avg_gain / avg_loss;//   (при avg_loss == 0 → RS → ∞)
    RSI = 100 - (100 / (1 + RS))
}
