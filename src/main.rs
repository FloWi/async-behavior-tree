use crate::behavior_tree::Behavior::{Action, Select, Sequence};
use crate::behavior_tree::Response::Success;
use crate::behavior_tree::{Actionable, Behavior, Response};
use async_trait::async_trait;
use serde::Serialize;
use tokio;

mod behavior_tree;

#[derive(Clone, Debug, Serialize)]
enum MyAction {
    Fail,
    Greet,
    Wave,
    Bow,
}

#[async_trait]
impl Actionable for MyAction {
    type ActionError = anyhow::Error;
    type ActionArgs = ();
    type ActionState = State;

    async fn run(
        &self,
        args: &Self::ActionArgs,
        state: &mut Self::ActionState,
    ) -> Result<Response, Self::ActionError> {
        match self {
            MyAction::Greet => {
                state.num_greets += 1;
                Ok(Success)
            }
            MyAction::Wave => {
                state.num_waves += 1;
                Ok(Success)
            }
            MyAction::Bow => {
                state.num_bows += 1;
                Ok(Success)
            }
            MyAction::Fail => {
                anyhow::bail!("Broken")
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)] // Add Clone derive
struct State {
    num_greets: u32,
    num_waves: u32,
    num_bows: u32,
}

#[tokio::main]
async fn main() {
    let bt: Behavior<MyAction> = Select(vec![
        Action(MyAction::Fail),
        Action(MyAction::Wave),
        Action(MyAction::Greet),
        Action(MyAction::Bow),
    ])
    .into();

    let mut my_state = State {
        num_greets: 0,
        num_waves: 0,
        num_bows: 0,
    };

    bt.run(&(), &mut my_state).await.unwrap();
    println!("{:?}", my_state);
    assert_eq!(
        my_state,
        State {
            num_greets: 0,
            num_waves: 1,
            num_bows: 0,
        }
    );

    let bt: Behavior<MyAction> = Sequence(vec![
        Action(MyAction::Wave),
        Action(MyAction::Greet),
        Action(MyAction::Fail),
        Action(MyAction::Bow),
    ])
    .into();

    let mut my_state = State {
        num_greets: 0,
        num_waves: 0,
        num_bows: 0,
    };

    let result = bt.run(&(), &mut my_state).await;
    println!("{:?}", my_state);
    assert_eq!(
        my_state,
        State {
            num_greets: 1,
            num_waves: 1,
            num_bows: 0,
        }
    );
}
