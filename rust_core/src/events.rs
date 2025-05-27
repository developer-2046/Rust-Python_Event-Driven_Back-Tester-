use crate::data::Tick;             

#[derive(Debug, Clone)]
pub enum Event {
    Market(Tick),
    Signal(SignalEvent),
    Order(OrderEvent),
    Fill(FillEvent),
}

#[derive(Debug, Clone)]
pub struct SignalEvent {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub long: bool,
}
#[derive(Debug, Clone)]
pub struct OrderEvent {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub long: bool,
    pub size: i32,
}
#[derive(Debug, Clone)]
pub struct FillEvent {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub long: bool,
    pub size: i32,
    pub price: f64,
    pub commission: f64,
}
