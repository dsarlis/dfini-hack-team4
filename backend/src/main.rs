use ic_cdk::api::caller;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{query, update};
use serde_bytes::ByteBuf;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

const INITIAL_TOKENS: Amount = 1000;

type AnswerId = u64;
type Content = ByteBuf;
type Duration = u64;
type TaskId = u64;
type TaskPayload = ByteBuf;
type Timestamp = u64;
type Amount = u64;

struct State {
    _tasks: RefCell<HashMap<TaskId, TaskInternal>>,
    _answers: RefCell<HashMap<AnswerId, Answer>>,
    ledger: RefCell<HashMap<Principal, Amount>>,
}

impl Default for State {
    fn default() -> Self {
        State {
            _tasks: RefCell::new(HashMap::default()),
            _answers: RefCell::new(HashMap::default()),
            ledger: RefCell::new(HashMap::default()),
        }
    }
}

thread_local! {
    static STATE: State = State::default();
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Choice {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Language {
    #[serde(rename = "chinese")]
    Chinese,
    #[serde(rename = "french")]
    French,
    #[serde(rename = "german")]
    German,
    #[serde(rename = "greek")]
    Greek,
    #[serde(rename = "hindi")]
    Hindi,
    #[serde(rename = "italian")]
    Italian,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TaskType {
    #[serde(rename = "translate_text")]
    TranslateText,
    #[serde(rename = "edit_image")]
    EditImage,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TaskStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TranslateTextInput {
    input: String,
    language: Language,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Answer {
    submitter: Principal,
    submission_time: Timestamp,
    votes: Vec<Vote>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Task {
    submitter: Principal,
    task_type: TaskType,
    payload: TaskPayload,
    deadline: Timestamp,
    reward: Amount,
    answers: Vec<Answer>,
    status: TaskStatus,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct ShortTask {
    id: TaskId,
    submitter: Principal,
    task_type: TaskType,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Vote {
    voter: Principal,
    choice: Choice,
}

#[allow(dead_code)]
struct TaskInternal {
    submitter: Principal,
    task_type: TaskType,
    payload: TaskPayload,
    deadline: Timestamp,
    reward: Amount,
    answers: HashSet<AnswerId>,
    status: TaskStatus,
}

#[update]
fn register() {
    let caller = caller();

    STATE.with(|s| {
        let mut ledger = s.ledger.borrow_mut();
        if ledger.contains_key(&caller) {
            ic_cdk::trap(&format!("{} has already registered.", caller));
        }
        ledger.insert(caller, INITIAL_TOKENS);
    });
}

#[update]
fn submit_task(
    _task_type: TaskType,
    _payload: TaskPayload,
    _duration: Duration,
    _reward: Amount,
) -> TaskId {
    0
}

#[query]
fn get_task(_id: TaskId) -> Task {
    Task {
        submitter: Principal::anonymous(),
        task_type: TaskType::TranslateText,
        payload: ByteBuf::from(vec![]),
        deadline: 0,
        reward: 0,
        answers: vec![],
        status: TaskStatus::Open,
    }
}

#[query]
fn get_all_tasks() -> Vec<ShortTask> {
    vec![]
}

#[query]
fn get_balance() -> Amount {
    0
}

#[update]
fn answer_task(_id: TaskId, _content: Content) -> AnswerId {
    0
}

#[update]
fn vote(_id: AnswerId, _choice: Choice) {}

#[export_name = "canister_heartbeat"]
fn hearbeat() {}

fn main() {}
