use ic_cdk::api::{caller, time};
use ic_cdk::export::candid::{CandidType, Decode, Deserialize, Principal};
use ic_cdk_macros::{query, update};
use serde_bytes::ByteBuf;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

// The initial amount of tokens given to a newly registered principal.
const INITIAL_TOKENS: Amount = 1000;

// The cost of submitting a task. The main purpose is to prevent DoS attacks.
const SUBMISSION_COST: Amount = 1;

// The minimum duration that a task can stay open.
const MIN_DURATION: std::time::Duration = std::time::Duration::from_secs(60); // 1 minute

// The maximum duration that a task can stay open.
const MAX_DURATION: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24); // 1 day

// The maximum size of the payload of a task.
const MAX_TASK_PAYLOAD: usize = 10 * 1024; // 100 KiB

type AnswerId = u64;
type Content = ByteBuf;
type Duration = u64;
type TaskId = u64;
type TaskPayload = ByteBuf;
type Timestamp = u64;
type Amount = u64;

struct State {
    next_task_id: RefCell<TaskId>,
    tasks: RefCell<HashMap<TaskId, TaskInternal>>,
    answers: RefCell<HashMap<AnswerId, Answer>>,
    ledger: RefCell<HashMap<Principal, Amount>>,
}

impl Default for State {
    fn default() -> Self {
        State {
            next_task_id: RefCell::new(0),
            tasks: RefCell::new(HashMap::default()),
            answers: RefCell::new(HashMap::default()),
            ledger: RefCell::new(HashMap::default()),
        }
    }
}

thread_local! {
    static STATE: State = State::default();
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
enum TaskType {
    #[serde(rename = "translate_text")]
    TranslateText,
    #[serde(rename = "edit_image")]
    EditImage,
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
struct Answer {
    submitter: Principal,
    submission_time: Timestamp,
    votes: Vec<Vote>,
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
struct Task {
    submitter: Principal,
    task_type: TaskType,
    payload: TaskPayload,
    deadline: Timestamp,
    reward: Amount,
    answers: Vec<Answer>,
    status: TaskStatus,
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
struct ShortTask {
    id: TaskId,
    submitter: Principal,
    task_type: TaskType,
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
struct Vote {
    voter: Principal,
    choice: Choice,
}

#[allow(dead_code)]
#[derive(Clone)]
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
    task_type: TaskType,
    payload: TaskPayload,
    duration: Duration,
    reward: Amount,
) -> TaskId {
    let caller = caller();

    STATE.with(|s| {
        let ledger = s.ledger.borrow();
        if !ledger.contains_key(&caller) {
            ic_cdk::trap(&format!("{} has not been registered yet.", caller));
        }

        let balance = *ledger.get(&caller).unwrap();
        if balance < SUBMISSION_COST {
            ic_cdk::trap(&format!(
                "{} has only {} tokens but {} are needed to submit a task.",
                caller, balance, SUBMISSION_COST
            ));
        }
    });

    match task_type {
        TaskType::TranslateText => {
            if payload.len() > MAX_TASK_PAYLOAD {
                ic_cdk::trap(&format!(
                    "Maximum size of payload is {} but {} was given.",
                    MAX_TASK_PAYLOAD,
                    payload.len()
                ));
            }

            if let Err(err) = Decode!(&payload, TranslateTextInput) {
                ic_cdk::trap(&format!("Invalid input for tranlating text: {}", err));
            };

            if duration < u64::try_from(MIN_DURATION.as_nanos()).unwrap() {
                ic_cdk::trap(&format!(
                    "Mininum duration for task is {:?}, but {:?} was given",
                    MIN_DURATION,
                    std::time::Duration::from_nanos(duration)
                ));
            }

            if duration > u64::try_from(MAX_DURATION.as_nanos()).unwrap() {
                ic_cdk::trap(&format!(
                    "Maximum duration for task is {:?}, but {:?} was given",
                    MAX_DURATION,
                    std::time::Duration::from_nanos(duration)
                ));
            }

            STATE.with(|s| {
                let mut ledger = s.ledger.borrow_mut();
                let balance = *ledger.get(&caller).unwrap();
                if balance < SUBMISSION_COST + reward {
                    ic_cdk::trap(&format!(
                        "{} has only {} tokens but {} were requested as a reward.",
                        caller,
                        balance - SUBMISSION_COST,
                        reward
                    ));
                }
                ledger.insert(caller, balance - SUBMISSION_COST - reward);
            });

            let task_id = STATE.with(|s| s.next_task_id.replace_with(|&mut old| old + 1));

            STATE.with(|s| {
                let mut tasks = s.tasks.borrow_mut();
                tasks.insert(
                    task_id,
                    TaskInternal {
                        submitter: caller,
                        task_type,
                        payload,
                        deadline: time() + duration,
                        reward,
                        answers: HashSet::new(),
                        status: TaskStatus::Open,
                    },
                );
            });

            task_id
        }
        TaskType::EditImage => ic_cdk::trap("Edit image use case is unimplemented!"),
    }
}

#[query]
fn get_task(id: TaskId) -> Task {
    let caller = caller();
    get_task_impl(caller, id)
}

fn get_task_impl(caller: Principal, id: TaskId) -> Task {
    let mut task: Task = Task {
        submitter: Principal::anonymous(),
        task_type: TaskType::TranslateText,
        payload: ByteBuf::from(vec![]),
        deadline: 0,
        reward: 0,
        answers: vec![],
        status: TaskStatus::Open,
    };

    STATE.with(|s| {
        let ledger = s.ledger.borrow();
        if !ledger.contains_key(&caller) {
            ic_cdk::trap(&format!("{} has not been registered yet.", caller));
        }
        let task_map = s.tasks.borrow();
        if !task_map.contains_key(&id) {
            ic_cdk::trap(&format!("Requested task id {} cannot be found", id));
        }
        let answers_map = s.answers.borrow();
        if let Some(task_internal_ref) = task_map.get(&id) {
            let task_internal : TaskInternal = (*task_internal_ref).clone();
            let mut answers:Vec<Answer> = Vec::new();
            for ans_id in task_internal.answers {
                if !answers_map.contains_key(&ans_id) {
                    ic_cdk::trap(&format!("Inconsistent state. AnswerId {} cannot be found", ans_id));
                }
                if let Some(ans_ref) = answers_map.get(&ans_id){
                    answers.push(ans_ref.clone());
                }
            }
            task = Task {
                submitter: task_internal.submitter,
                task_type: task_internal.task_type,
                payload: task_internal.payload,
                deadline: task_internal.deadline,
                reward: task_internal.reward,
                answers,
                status: task_internal.status,
            };
        }

    });
    task
}

#[query]
fn get_all_tasks() -> Vec<ShortTask> {
    let caller = caller();
    let mut tasks = Vec::new();

    STATE.with(|s| {
        let ledger = s.ledger.borrow();
        if !ledger.contains_key(&caller) {
            ic_cdk::trap(&format!("{} has not been registered yet.", caller));
        }
        let task_map = s.tasks.borrow();
        for (task_id_ref, task_internal_ref) in task_map.iter() {
            let task_internal= (*task_internal_ref).clone();
            tasks.push(ShortTask {
                id: task_id_ref.clone(),
                submitter: task_internal.submitter,
                task_type: task_internal.task_type
            })
        }
    });

    tasks
}

#[query]
fn get_balance() -> Amount {
    let caller = caller();
    let mut amount : u64 = 0;

    STATE.with(|s| {
        let ledger = s.ledger.borrow();
        if !ledger.contains_key(&caller) {
            ic_cdk::trap(&format!("{} has not been registered yet.", caller));
        }
        if let Some(amt) = ledger.get(&caller) {
            amount = *amt
        }
    });

    amount
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_task() {
        let principal1 = Principal::from_text("ollzc-44u63-k7twc-5fffp-f5vr7-ptyb5-v6esl-cswqt-f4u4t-qfxbj-6ae").unwrap();
        let principal2 = Principal::from_text("etn6u-fsyqb-qmjjs-murc2-w4qtf-4tota-tp2vi-y5jau-kavt5-zi6dj-lae").unwrap();
        let principal3 = Principal::from_text("lv3pe-37kt2-3kbhe-r2oyg-ppn4z-mftmh-j242j-gu7ps-hvun5-3ioxb-3qe").unwrap();

        let mut v = Vec::new();
        v.push(Vote {
            voter: Principal::anonymous(),
            choice: Choice::Yes,
        });
        v.push(Vote{
            voter: Principal::anonymous(),
            choice:Choice::Yes,
        });

        let mut answers_map = HashMap::new();
        let id = 1;
        answers_map.insert(id, Answer {
            submitter: principal1,
            submission_time: 1631075074,
            votes: v.clone(),
        });
        answers_map.insert(id + 1, Answer {
            submitter: principal1,
            submission_time: 1631075073,
            votes: v.clone(),
        });
        answers_map.insert(id + 2, Answer {
            submitter: principal2,
            submission_time: 1631075074,
            votes: v.clone(),
        });
        answers_map.insert(id + 3, Answer {
            submitter: principal2,
            submission_time: 1631075073,
            votes: v.clone(),
        });

        //let mut tasks = HashMap::new();
        let bytes: [u8; 7] = [65, 66, 67, 68, 69, 70, 71];
        let mut ans_ids: HashSet<u64> = HashSet::new();
        ans_ids.insert(1);
        ans_ids.insert(2);
        ans_ids.insert(3);
        ans_ids.insert(4);

        super::STATE.with(|s| {
            //s.tasks = RefCell::new(tasks.clone());
            //s.answers = RefCell::new(answers_map.clone());
            //s.ledger = RefCell::new(ledger.clone());
            let mut ledger = s.ledger.borrow_mut();
            ledger.insert(principal1, 100);
            ledger.insert(principal2, 200);

            let mut answers_map = s.answers.borrow_mut();
            answers_map.insert(id, Answer {
                submitter: principal1,
                submission_time: 1631075074,
                votes: v.clone(),
            });
            answers_map.insert(id + 1, Answer {
                submitter: principal1,
                submission_time: 1631075073,
                votes: v.clone(),
            });
            answers_map.insert(id + 2, Answer {
                submitter: principal2,
                submission_time: 1631075074,
                votes: v.clone(),
            });
            answers_map.insert(id + 3, Answer {
                submitter: principal2,
                submission_time: 1631075073,
                votes: v.clone(),
            });

            let mut tasks = s.tasks.borrow_mut();
            tasks.insert(id, TaskInternal {
                submitter: principal1,
                task_type: TaskType::TranslateText,
                payload: ByteBuf::from(bytes),
                deadline: 1631075080,
                reward: 12,
                answers: ans_ids.clone(),
                status: TaskStatus::Open,
            });
            tasks.insert(id + 1, TaskInternal {
                submitter: principal1,
                task_type: TaskType::TranslateText,
                payload: ByteBuf::from(bytes),
                deadline: 1631075083,
                reward: 11,
                answers: ans_ids.clone(),
                status: TaskStatus::Open,
            });
            tasks.insert(id + 2, TaskInternal {
                submitter: principal2,
                task_type: TaskType::TranslateText,
                payload: ByteBuf::from(bytes),
                deadline: 1631075085,
                reward: 10,
                answers: ans_ids.clone(),
                status: TaskStatus::Open,
            });
        });

        // Request from unregistered principal
        let result = std::panic::catch_unwind(||get_task_impl(principal3, 1));
        assert!(result.is_err());

        // Request for valid task
        let mut answers : Vec<Answer> = Vec::new();
        for ans_id in ans_ids {
            if let Some(ans_ref) = answers_map.get(&ans_id) {
                answers.push(ans_ref.clone());
            }
        }
        let expected_result = Task {
            submitter: principal1,
            task_type: TaskType::TranslateText,
            payload: ByteBuf::from(bytes),
            deadline: 1631075080,
            reward: 12,
            answers,
            status: TaskStatus::Open,
        };
        let result = get_task_impl(principal1, 1);
        assert_eq!(result, expected_result);

        // Request for invalid task
        let result = std::panic::catch_unwind(||get_task_impl(principal3, 10));
        assert!(result.is_err());

    }
}