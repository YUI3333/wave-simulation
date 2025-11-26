use serde::Serialize;
use std::fs;

// 通用模拟参数
const DX: f64 = 0.01; // 空间步长（m）
const L: f64 = 1.0;   // 弦长（m）
const NUM_NODES: usize = (L / DX) as usize + 1; // 节点数（0~100，共101个）

/// 第一部分：单一r值的弦振动模拟
fn simulate_single_r(r: f64, num_steps: usize) -> Vec<Vec<f64>> {
    let mut y_prev_prev = vec![0.0; NUM_NODES]; // n-1时刻位移
    let mut y_prev = vec![0.0; NUM_NODES];     // n时刻位移
    let mut y_curr = vec![0.0; NUM_NODES];     // n+1时刻位移
    let mut results = Vec::with_capacity(num_steps);

    // 初始条件：n=0（高斯脉冲，中心x=0.2m，振幅0.5）- 明确所有浮点变量类型为f64
    let center: f64 = 0.2;
    let sigma: f64 = 0.05;
    let amplitude: f64 = 0.5;
    for (i, y) in y_prev_prev.iter_mut().enumerate() {
        let x = i as f64 * DX;
        *y = amplitude * (-((x - center).powi(2)) / (2.0 * sigma.powi(2))).exp();
    }

    // n=1时刻：初始速度为0 → y_prev = y_prev_prev
    y_prev.copy_from_slice(&y_prev_prev);
    results.push(y_prev_prev.clone());
    results.push(y_prev.clone());

    // 迭代计算时间步
    let r_sq = r.powi(2);
    for _ in 2..num_steps {
        // 端点固定
        y_curr[0] = 0.0;
        y_curr[NUM_NODES - 1] = 0.0;

        // 内部节点计算
        for i in 1..NUM_NODES - 1 {
            y_curr[i] = 2.0 * (1.0 - r_sq) * y_prev[i] 
                - y_prev_prev[i] 
                + r_sq * (y_prev[i + 1] + y_prev[i - 1]);
        }

        results.push(y_curr.clone());
        // 更新前两时刻数据
        y_prev_prev.copy_from_slice(&y_prev);
        y_prev.copy_from_slice(&y_curr);
    }

    results
}

/// 第二部分：波在界面的反射与折射模拟（多r值）
fn simulate_interface(r_values: &[f64], num_steps: usize) -> Vec<Vec<Vec<f64>>> {
    const C1: f64 = 300.0; // 左半区波速（m/s）
    const C2: f64 = 150.0; // 右半区波速（m/s）
    const INTERFACE_I: usize = 50; // 界面节点（x=0.5m）
    let mut all_results = Vec::with_capacity(r_values.len());

    for &r1 in r_values {
        let dt = r1 * DX / C1; // 时间步长（左半区Courant数=r1）
        let r1_sq = r1.powi(2);
        let r2 = (C2 * dt) / DX; // 右半区Courant数
        let r2_sq = r2.powi(2);

        let mut y_prev_prev = vec![0.0; NUM_NODES];
        let mut y_prev = vec![0.0; NUM_NODES];
        let mut y_curr = vec![0.0; NUM_NODES];
        let mut results = Vec::with_capacity(num_steps);

        // 初始条件：左半区高斯脉冲（中心x=0.1m）- 明确所有浮点变量类型为f64
        let center: f64 = 0.1;
        let sigma: f64 = 0.05;
        let amplitude: f64 = 0.5;
        for (i, y) in y_prev_prev.iter_mut().enumerate() {
            let x = i as f64 * DX;
            *y = amplitude * (-((x - center).powi(2)) / (2.0 * sigma.powi(2))).exp();
        }

        y_prev.copy_from_slice(&y_prev_prev);
        results.push(y_prev_prev.clone());
        results.push(y_prev.clone());

        // 迭代计算
        for _ in 2..num_steps {
            // 端点固定
            y_curr[0] = 0.0;
            y_curr[NUM_NODES - 1] = 0.0;

            // 左半区（i=1~49）
            for i in 1..INTERFACE_I {
                y_curr[i] = 2.0 * (1.0 - r1_sq) * y_prev[i] 
                    - y_prev_prev[i] 
                    + r1_sq * (y_prev[i + 1] + y_prev[i - 1]);
            }

            // 右半区（i=51~99）
            for i in INTERFACE_I + 1..NUM_NODES - 1 {
                y_curr[i] = 2.0 * (1.0 - r2_sq) * y_prev[i] 
                    - y_prev_prev[i] 
                    + r2_sq * (y_prev[i + 1] + y_prev[i - 1]);
            }

            // 界面节点（i=50）：简化处理（位移连续）
            y_curr[INTERFACE_I] = 2.0 * (1.0 - (r1_sq + r2_sq) / 2.0) * y_prev[INTERFACE_I]
                - y_prev_prev[INTERFACE_I]
                + (r1_sq * y_prev[INTERFACE_I - 1] + r2_sq * y_prev[INTERFACE_I + 1]) / 2.0;

            results.push(y_curr.clone());
            y_prev_prev.copy_from_slice(&y_prev);
            y_prev.copy_from_slice(&y_curr);
        }

        all_results.push(results);
    }

    all_results
}

/// 生成单一r值的波形HTML（含动画）
fn generate_single_r_html(r: f64, results: &[Vec<f64>], filename: &str) {
    let x_data: Vec<f64> = (0..NUM_NODES).map(|i| i as f64 * DX).collect();
    let x_json = serde_json::to_string(&x_data).unwrap();
    let wave_json = serde_json::to_string(results).unwrap();

    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Wave Simulation (r={})</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>canvas {{width:800px;height:400px;margin:20px auto;display:block;}}</style>
</head>
<body>
    <h1 style="text-align:center">Waveform with r = {}</h1>
    <canvas id="waveChart"></canvas>
    <script>
        const ctx = document.getElementById('waveChart').getContext('2d');
        const xData = {};
        const waveData = {};
        let step = 0;

        const chart = new Chart(ctx, {{
            type: 'line',
            data: {{
                labels: xData,
                datasets: [{{label: 't='+step+'Δt', data: waveData[step], borderColor: 'blue', borderWidth:2, fill:false}}]
            }},
            options: {{
                scales: {{
                    x: {{title: {{display:true, text:'Position (m)'}}, min:0, max:1.0}},
                    y: {{title: {{display:true, text:'Displacement (m)'}}, min:-0.6, max:0.6}}
                }},
                animation: {{duration:0}}
            }}
        }});

        setInterval(() => {{
            step = (step + 1) % waveData.length;
            chart.data.datasets[0].data = waveData[step];
            chart.data.datasets[0].label = 't='+step+'Δt';
            chart.update();
        }}, 50);
    </script>
</body>
</html>
    "#, r, r, x_json, wave_json);

    fs::write(filename, html).unwrap();
    println!("Generated: {}", filename);
}

/// 生成界面反射折射的HTML（多r值对比）
fn generate_interface_html(r_values: &[f64], all_results: &[Vec<Vec<f64>>], filename: &str) {
    let x_data: Vec<f64> = (0..NUM_NODES).map(|i| i as f64 * DX).collect();
    let x_json = serde_json::to_string(&x_data).unwrap();
    let wave_json = serde_json::to_string(all_results).unwrap();
    let interface_x = 0.5;

    // 为不同r值分配颜色和数据集
    let datasets: Vec<String> = r_values.iter().enumerate().map(|(idx, &r)| {
        format!(r#"{{
            label: 'r={}',
            data: waveData[{}][0],
            borderColor: '{}',
            borderWidth: 2,
            fill: false
        }}"#, r, idx, get_color(idx))
    }).collect();

    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Wave Reflection & Refraction</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        canvas {{width:800px;height:400px;margin:20px auto;display:block;}}
        .control {{text-align:center;margin:10px;}}
        button {{padding:5px 10px;margin:0 5px;}}
    </style>
</head>
<body>
    <h1 style="text-align:center">Reflection & Refraction (c₁=300m/s, c₂=150m/s)</h1>
    <div class="control">
        <button onclick="prevStep()">← Previous</button>
        <button onclick="nextStep()">Next →</button>
        <span id="stepLabel">t=0Δt</span>
    </div>
    <canvas id="waveChart"></canvas>
    <script>
        const ctx = document.getElementById('waveChart').getContext('2d');
        const xData = {};
        const waveData = {};
        const interfaceX = {};
        let step = 0;
        const maxStep = waveData[0].length - 1;

        const chart = new Chart(ctx, {{
            type: 'line',
            data: {{labels: xData, datasets: [{}]}},
            options: {{
                scales: {{
                    x: {{title: {{display:true, text:'Position (m)'}}, min:0, max:1.0,
                        ticks: {{callback: v => v === interfaceX ? v+' (interface)' : v}}}},
                    y: {{title: {{display:true, text:'Displacement (m)'}}, min:-0.6, max:0.6}}
                }},
                animation: {{duration:200}}
            }}
        }});

        function updateChart() {{
            document.getElementById('stepLabel').textContent = 't='+step+'Δt';
            waveData.forEach((data, idx) => chart.data.datasets[idx].data = data[step]);
            chart.update();
        }}
        function nextStep() {{if (step < maxStep) {{step++; updateChart();}}}}
        function prevStep() {{if (step > 0) {{step--; updateChart();}}}}
    </script>
</body>
</html>
    "#, x_json, wave_json, interface_x, datasets.join(", "));

    fs::write(filename, html).unwrap();
    println!("Generated: {}", filename);
}

/// 为不同r值分配颜色
fn get_color(idx: usize) -> &'static str {
    match idx {0 => "red", 1 => "green", 2 => "blue", _ => "black"}
}

fn main() {
    // 第一部分：生成3个单一r值的波形HTML
    let r_single = [0.8, 1.0, 1.2];
    let num_steps_single = 150;
    for &r in &r_single {
        let results = simulate_single_r(r, num_steps_single);
        let filename = format!("wave_r{:.1}.html", r);
        generate_single_r_html(r, &results, &filename);
    }

    // 第二部分：生成界面反射折射的HTML
    let r_interface = [0.6, 0.8, 1.0];
    let num_steps_interface = 200;
    let all_results = simulate_interface(&r_interface, num_steps_interface);
    generate_interface_html(&r_interface, &all_results, "wave_interface.html");
}