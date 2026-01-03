use crate::agent::{Routine, RoutineId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 跟踪所有活跃的 Routine 及其层级关系
pub struct RoutineManager {
    routines: Arc<RwLock<HashMap<RoutineId, Routine>>>,
}

impl Default for RoutineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutineManager {
    pub fn new() -> Self {
        Self {
            routines: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册新的 Routine
    pub fn register(&self, routine: Routine) {
        let mut routines = self.routines.write().unwrap();
        routines.insert(routine.id, routine);
    }

    /// 获取 Routine
    pub fn get(&self, id: &RoutineId) -> Option<Routine> {
        let routines = self.routines.read().unwrap();
        routines.get(id).cloned()
    }

    /// 获取所有 Routine 的数量
    pub fn count(&self) -> usize {
        let routines = self.routines.read().unwrap();
        routines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Routine;
    use uuid::Uuid;

    #[test]
    fn test_routine_manager() {
        let manager = RoutineManager::new();
        let routine = Routine::new(Uuid::new_v4());
        let id = routine.id;

        manager.register(routine);
        assert_eq!(manager.count(), 1);
        assert!(manager.get(&id).is_some());
    }
}
