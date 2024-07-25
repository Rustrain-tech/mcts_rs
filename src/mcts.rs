use rand::prelude::*;

/// モンテカルロ木探索の流れ
/// モンテカルロ木探索は以下の4つのステップで構成される
/// 1. 選択(select): UCTアルゴリズムを用いてノードを辿り、未展開のノードに到達したらそのノードを選択する
/// 2. 拡張(expand): 未展開のノードであれば、そのノードを展開し、そのノードの子ノードを作成する
/// 3. シミュレーション(simulate): 新たに作成したノードからプレイアウトを行い、その結果を得る
/// 4. 伝播(backpropagate): プレイアウトの結果を新たに作成したノードから根ノードまで伝播し、各ノードの勝利回数と訪問回数を更新する
///

pub type NodeIndex = usize;
pub type Action = usize;

pub struct MCTS {
    root: NodeIndex,  // ルートノードのインデックス
    nodes: Vec<Node>, // ノードのリスト
}

pub struct Node {
    state: Box<dyn GameState>,    // ゲームの状態
    parent: Option<NodeIndex>,    // 親ノードのインデックス
    children: Vec<NodeIndex>,     // 子ノードのインデックス
    wins: f64,                    // 勝利回数
    visits: f64,                  // 訪問回数
    untried_actions: Vec<Action>, // 未試行の行動
    last_action: Option<Action>,  // 最後に選択された行動
}

/// 問題ごとに実装する
pub trait GameState {
    fn get_legal_moves(&self) -> Vec<Action>; // 合法手のリストを返す
    fn make_move(&mut self, action: Action); // 手を打つ
    fn is_terminal(&self) -> bool; // ゲームが終了しているかどうか
    fn get_winner(&self) -> Option<i32>; // 勝者を返す
    fn clone(&self) -> Box<dyn GameState>; // ゲームの状態を複製する
}

impl MCTS {
    // ゲームの情報を受け取ってMCTSを初期化する
    pub fn new(state: Box<dyn GameState>) -> MCTS {
        let root: NodeIndex = 0;
        let nodes: Vec<Node> = vec![Node {
            state: state.clone(),
            parent: None,
            children: vec![],
            wins: 0.0,
            visits: 0.0,
            untried_actions: state.get_legal_moves(),
            last_action: None,
        }];
        MCTS { root, nodes }
    }

    /// 引数のノードからUCTアルゴリズムを用いて到達した葉ノードを返す
    pub fn select(&self, node_index: NodeIndex) -> NodeIndex {
        // 未展開のノードなのでそのノードを返す
        if !self.nodes[node_index].untried_actions.is_empty() {
            return node_index;
        }
        self.nodes[node_index]
            .children
            .iter()
            .max_by(|&&a, &&b| {
                let node_a = &self.nodes[a];
                let node_b = &self.nodes[b];
                let uct_a = node_a.wins / node_a.visits
                    + (2.0 * (self.nodes[node_index].visits).ln() / node_a.visits).sqrt();
                let uct_b = node_b.wins / node_b.visits
                    + (2.0 * (self.nodes[node_index].visits).ln() / node_b.visits).sqrt();
                uct_a.partial_cmp(&uct_b).unwrap()
            })
            .map(|&child| self.select(child))
            .unwrap_or(node_index)
    }

    /// 引数のノードから有効な子ノードを1つ選択する
    pub fn expand(&mut self, node_index: usize) -> NodeIndex {
        if !self.nodes[node_index].untried_actions.is_empty() {
            let action = self.nodes[node_index].untried_actions.pop().unwrap();
            let mut state = self.nodes[node_index].state.clone();
            state.make_move(action);
            let new_node_index = self.nodes.len();
            let untried_actions = state.get_legal_moves();
            self.nodes.push(Node {
                state,
                parent: Some(node_index),
                children: vec![],
                wins: 0.0,
                visits: 0.0,
                untried_actions,
                last_action: Some(action),
            });
            self.nodes[node_index].children.push(new_node_index);
            new_node_index
        } else {
            self.nodes[node_index]
                .children
                .iter()
                .max_by(|&&a, &&b| {
                    let node_a = &self.nodes[a];
                    let node_b = &self.nodes[b];
                    let uct_a = node_a.wins / node_a.visits
                        + (2.0 * (self.nodes[node_index].visits).ln() / node_a.visits).sqrt();
                    let uct_b = node_b.wins / node_b.visits
                        + (2.0 * (self.nodes[node_index].visits).ln() / node_b.visits).sqrt();
                    uct_a.partial_cmp(&uct_b).unwrap()
                })
                .copied()
                .unwrap_or(node_index)
        }
    }

    /// プレイアウトを行い、その結果を返す
    pub fn simulate(&self, node_index: usize) -> f64 {
        let mut state = self.nodes[node_index].state.clone();
        let mut rng = thread_rng();
        while !state.is_terminal() {
            let legal_moves = state.get_legal_moves();
            let move_ = legal_moves[rng.gen_range(0..legal_moves.len())];
            state.make_move(move_);
        }
        match state.get_winner() {
            Some(0) => 0.5,
            Some(1) => 1.0,
            Some(-1) => 0.0,
            _ => panic!("Unexpected winner"),
        }
    }

    /// プレイアウトの結果を伝播する
    pub fn backpropagate(&mut self, node_index: NodeIndex, result: f64) {
        self.nodes[node_index].visits += 1.0;
        self.nodes[node_index].wins += result;
        if let Some(parent) = self.nodes[node_index].parent {
            self.backpropagate(parent, result);
        }
    }

    /// 指定された回数のシミュレーションを行い最適な手を返す
    pub fn get_best_move(&mut self, iterations: u32) -> Action {
        for _ in 0..iterations {
            let selected_node: NodeIndex = self.select(self.root);
            let expanded_node: NodeIndex = self.expand(selected_node);
            if expanded_node == selected_node {
                break;
            }
            let result = self.simulate(expanded_node);
            self.backpropagate(expanded_node, result);
        }

        let best_action = self.nodes[self.root]
            .children
            .iter()
            .max_by(|&&a, &&b| {
                let node_a = &self.nodes[a];
                let node_b = &self.nodes[b];
                let uct_a = node_a.visits;
                let uct_b = node_b.visits;
                uct_a.partial_cmp(&uct_b).unwrap()
            })
            .map(|&child| self.nodes[child].last_action.unwrap())
            .unwrap_or_else(|| panic!("Failed to get best move"));

        best_action
    }
}
