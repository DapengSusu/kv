use crate::*;

impl CommandService for Hset {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        match self.pair {
            Some(v) => match store.set(&self.table, v.key, v.value.unwrap_or_default()) {
                Ok(Some(v)) => v.into(),
                Ok(None) => Value::default().into(),
                Err(e) => e.into(),
            },
            None => Value::default().into(),
        }
    }
}

impl CommandService for Hget {
    fn execute(self, store: &impl crate::Storage) -> CommandResponse {
        match store.get(&self.table, &self.key) {
            Ok(Some(v)) => v.into(),
            Ok(None) => KVError::NotFound(self.table, self.key).into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hgetall {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        match store.get_all(&self.table) {
            Ok(v) => v.into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hdel {
    fn execute(self, store: &impl Storage) -> CommandResponse {
        match store.del(&self.table, &self.key) {
            Ok(Some(v)) => v.into(),
            Ok(None) => KVError::NotFound(self.table, self.key).into(),
            Err(e) => e.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use command_request::RequestData;

    #[test]
    fn hset_should_work() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hset("t1", "hello", "world".into());
        let res = dispatch(cmd.clone(), &store);
        assert_res_ok(res, &[Value::default()], &[]);

        let res = dispatch(cmd, &store);
        assert_res_ok(res, &["world".into()], &[]);
    }

    #[test]
    fn hget_should_work() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hset("score", "u1", 10.into());
        let _ = dispatch(cmd, &store);

        let cmd = CommandRequest::new_hget("score", "u1");
        let res = dispatch(cmd, &store);

        assert_res_ok(res, &[10.into()], &[]);
    }

    #[test]
    fn hget_with_non_exist_key_should_return_404() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hget("score", "u1");
        let res = dispatch(cmd, &store);

        assert_res_error(res, 404, "Not found");
    }

    #[test]
    fn hgetall_should_work() {
        let store = MemTable::new();
        let cmds = vec![
            CommandRequest::new_hset("score", "u1", 10.into()),
            CommandRequest::new_hset("score", "u2", 8.into()),
            CommandRequest::new_hset("score", "u3", 11.into()),
            CommandRequest::new_hset("score", "u1", 6.into())
        ];

        for cmd in cmds {
            dispatch(cmd, &store);
        }

        let cmd = CommandRequest::new_hgetall("score");
        let res = dispatch(cmd, &store);
        let pairs = &[
            KvPair::new("u1", 6.into()),
            KvPair::new("u2", 8.into()),
            KvPair::new("u3", 11.into())
        ];

        assert_res_ok(res, &[], pairs);
    }

    #[test]
    fn hdel_should_work() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hset("score", "u1", 60.into());
        let _ = dispatch(cmd, &store);
        let cmd = CommandRequest::new_hset("score", "u2", 74.into());
        let _ = dispatch(cmd, &store);

        let cmd = CommandRequest::new_hdel("score", "u1");
        let res = dispatch(cmd, &store);

        assert_res_ok(res, &[60.into()], &[]);
    }

    #[test]
    fn hdel_with_non_exist_key_should_return_404() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hset("score", "u1", 56.into());
        let _ = dispatch(cmd, &store);

        let cmd = CommandRequest::new_hdel("score", "u2");
        let res = dispatch(cmd, &store);

        assert_res_error(res, 404, "Not found");
    }

    // 从 Request 中得到 Response，目前处理 HSET，HGET，HGETALL，HDEL
    fn dispatch(cmd: CommandRequest, store: &impl Storage) -> CommandResponse {
        match cmd.request_data.unwrap() {
            RequestData::Hset(v) => v.execute(store),
            RequestData::Hget(v) => v.execute(store),
            RequestData::Hgetall(v) => v.execute(store),
            RequestData::Hdel(v) => v.execute(store),
            _ => todo!(), // TODO
        }
    }

    // 测试成功返回的结果
    fn assert_res_ok(mut res: CommandResponse, values: &[Value], pairs: &[KvPair]) {
        res.pairs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(res.status, 200);
        assert_eq!(res.message, "");
        assert_eq!(res.values, values);
        assert_eq!(res.pairs, pairs);
    }

    // 测试失败返回的结果
    fn assert_res_error(res: CommandResponse, code: u32, msg: &str) {
        assert_eq!(res.status, code);
        assert!(res.message.contains(msg));
        assert_eq!(res.values, &[]);
        assert_eq!(res.pairs, &[]);
    }
}
