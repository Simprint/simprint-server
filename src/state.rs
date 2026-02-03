use uuid::Uuid;

/// 当前用户
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_uuid: Uuid,
}

/// 当前工作空间
#[derive(Debug, Clone)]
pub struct CurrentWorkspace {
    pub workspace_uuid: Uuid,
}

/// 当前 IP 地址
#[derive(Debug, Clone)]
pub struct CurrentIpAddr {
    pub real_ip: String,
}

/// 当前机器用户
#[derive(Debug, Clone)]
pub struct CurrentMachineUser {
    pub machine_code: String,
    pub platform: Option<String>,
    pub user_uuid: Option<uuid::Uuid>,
    pub is_allowed: bool, // 是否允许使用
}

/// 请求上下文 - 包含所有请求相关的上下文信息
///
/// 在中间件中逐步填充，handler 中通过 Extension 获取
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    /// 当前用户信息
    pub current_user: Option<CurrentUser>,
    /// 当前 IP 地址
    pub current_ip_addr: Option<CurrentIpAddr>,
    /// 当前机器用户
    pub current_machine_user: Option<CurrentMachineUser>,
    /// 当前团队 UUID
    pub current_team_uuid: Option<Uuid>,
    /// 当前工作空间 UUID
    pub current_workspace_uuid: Option<Uuid>,
}

impl RequestContext {
    /// 获取用户 UUID（如果已认证）
    pub fn user_uuid(&self) -> Option<Uuid> {
        self.current_user.as_ref().map(|u| u.user_uuid)
    }

    /// 获取用户 UUID，如果未认证则 panic
    pub fn user_uuid_unwrap(&self) -> Uuid {
        self.current_user.as_ref().expect("用户未认证").user_uuid
    }

    /// 获取 IP 地址
    pub fn ip(&self) -> Option<&str> {
        self.current_ip_addr.as_ref().map(|i| i.real_ip.as_str())
    }

    /// 获取 IP 地址，如果不存在则返回 "unknown"
    pub fn ip_or_unknown(&self) -> &str {
        self.ip().unwrap_or("unknown")
    }

    /// 获取工作空间 UUID
    pub fn workspace_uuid(&self) -> Option<Uuid> {
        self.current_workspace_uuid
    }

    /// 获取工作空间 UUID，如果不存在则 panic
    pub fn workspace_uuid_unwrap(&self) -> Uuid {
        self.current_workspace_uuid.expect("工作空间未设置")
    }
}
