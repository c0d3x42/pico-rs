use serde::{
    de::{Error, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::commands::action::Action;
use crate::commands::execution::{ActionExecution, ActionResult, ActionValue, ConditionExecution};
use crate::conditions::Condition;
use crate::context::PicoContext;
//use crate::state::PicoState;
use crate::rules::PicoRules;
use crate::runtime::PicoRuntime;

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct StopCommand {
    stop: String,
}
impl ActionExecution for StopCommand {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ActionResult {
        debug!("stopping because {:?}", self.stop);
        Ok(ActionValue::Stop(Some(self.stop.clone())))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BreakToCommand {
    r#break: uuid::Uuid,
}
impl ActionExecution for BreakToCommand {
    fn run_with_context(
        &self,
        _pico_rules: &PicoRules,
        _runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ActionResult {
        debug!("breaking to {:?}", self.r#break);
        Ok(ActionValue::BreakTo(self.r#break))
    }
}

fn multiple_if_then_elseif_tuples<'de, D>(deserializer: D) -> Result<MifThenElseBlock, D::Error>
where
    D: Deserializer<'de>,
{
    info!("inside");

    struct MifVisitor;
    impl<'de> Visitor<'de> for MifVisitor {
        type Value = MifThenElseBlock;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a nonempty sequence of a sequence of if_then_elseif_else")
        }

        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            Ok(MifThenElseBlock {
                if_thens: Vec::new(),
                then_else: None,
            })
        }
    }

    deserializer.deserialize_any(MifVisitor)
}

#[derive(Debug)]
struct MifVisitor;
impl<'de> Visitor<'de> for MifVisitor {
    type Value = MifThenElseBlock;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("array of condition..action")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let m = MifThenElseBlock {
            if_thens: Vec::new(),
            then_else: None,
        };

        info!("SELF {:?}", self);

        if let Some(first_if) = seq.next_element::<Condition>()? {
            if let Some(first_then) = seq.next_element::<Action>()? {
                info!("got first if then");
            }
        }

        loop {
            let c = seq.next_element::<Condition>();
            info!("C = {:?}", c);
            if c.is_err() {
                error!("C {:?}", c);
                break;
            }
            let cc = c.unwrap();
            if cc.is_none() {
                break;
            }
            if let Some(a) = seq.next_element::<Action>()? {
                info!("action followed");
            }
        }

        while let Some(cond) = seq.next_element::<Condition>().ok() {
            info!("Some cond {:?}", cond);
            if cond.is_none() {
                break;
            }

            if let Some(act) = seq.next_element::<Action>()? {
                info!("CA pair {:?},{:?}", cond, act);
            } else {
                warn!("non action followed condition");
                return Err(Error::custom(format!("unexpected c/a")));
            }
        }
        info!("Checking for else");
        if let Some(els) = seq.next_element::<Action>().ok() {
            info!("got an else {:?}", els);
        }

        /*
                info!("visit ");
                let c = seq.next_element::<Condition>()?;
                info!("visit2 {:?} ", c);

                let a = seq.next_element::<Action>()?;
                info!("visit3 {:?} ", a);
        */
        //while let Some(k) = seq.next_element()? {
        //    info!("consuming k");
        // }
        Ok(m)
    }
}

#[derive(Deserialize, Debug)]
struct MifThen(Condition, Action);

#[derive(Serialize, Debug)]
pub struct MifThenElseBlock {
    if_thens: Vec<(Condition, Action)>,
    then_else: Option<Action>,
}

impl<'de> Deserialize<'de> for MifThenElseBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(MifVisitor)
        //info!("{:?}", deserializer);
        /*
        Ok(MifThenElseBlock {
            if_thens: Vec::new(),
            then_else: None,
        })
        */
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MifThenElse {
    //#[serde(deserialize_with = "multiple_if_then_elseif_tuples")]
    mif: MifThenElseBlock,
}

impl ActionExecution for MifThenElse {
    fn run_with_context(
        &self,
        _pico_rule: &PicoRules,
        _runtime: &PicoRuntime,
        _ctx: &mut PicoContext,
    ) -> ActionResult {
        debug!("Mif {:?}", self);
        Ok(ActionValue::Continue)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IfThenElse {
    r#if: Condition,
    r#then: Action,
    r#else: Option<Action>,

    #[serde(default = "IfThenElse::default_uuid")]
    uuid: uuid::Uuid,
}
impl IfThenElse {
    fn default_uuid() -> uuid::Uuid {
        trace!("assigning default uuid");
        Uuid::new_v4()
    }
}

impl ActionExecution for IfThenElse {
    fn run_with_context(
        &self,
        pico_rules: &PicoRules,
        runtime: &PicoRuntime,
        ctx: &mut PicoContext,
    ) -> ActionResult {
        info!("running ITE -> {:?}", self.uuid);
        let if_result: bool = self.r#if.run_with_context(pico_rules, runtime, ctx)?;

        match if_result {
            true => self.then.run_with_context(pico_rules, runtime, ctx),
            false => match &self.r#else {
                None => Ok(ActionValue::Continue),
                Some(else_branch) => else_branch.run_with_context(pico_rules, runtime, ctx),
            },
        }
    }
}
