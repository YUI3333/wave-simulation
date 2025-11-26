use std::fs::write;
use serde_json::to_string;

fn main() {
    // 核心物理参数（优化后演示效果更佳）
    const L: f64 = 12.0;          // 弦长（略长，避免波包快速反射）
    const N: usize = 150;         // 空间采样点（更多点让波形更平滑）
    const T_STEPS: usize = 200;   // 时间步数（足够展示波包单向传播全程）
    const C: f64 = 1.2;           // 波速（适中，避免过快或过慢）
    const R: f64 = 0.8;           // Courant数（<1保证稳定，优化数值精度）
    const A: f64 = 1.0;           // 波包振幅（视觉清晰）
    const X0: f64 = 3.0;          // 初始位置（居中偏左，避免靠近边界）
    const SIGMA: f64 = 1.2;       // 波包宽度（饱满不纤细，传播时形状不变）

    // 基础步长计算
    let dx = L / (N - 1) as f64;
    let dt = R * dx / C;

    // 初始化位移数组（t-Δt、t、t+Δt）
    let mut u_prev = vec![0.0; N];
    let mut u_curr = vec![0.0; N];
    let mut u_next = vec![0.0; N];
    let mut frames = Vec::with_capacity(T_STEPS);

    // 设置初始条件（核心：匹配行波的位移和速度，确保单向传播不分裂）
    for i in 0..N {
        let x = i as f64 * dx;
        // 初始位移：高斯波包 u(x,0) = A·exp(-(x-x0)²/σ²)
        let u0 = A * (-((x - X0).powi(2)) / SIGMA.powi(2)).exp();
        u_curr[i] = u0;

        // 初始速度：v(x,0) = -c·u’(x,0)（行波条件，保证单向传播）
        let du_dx = u0 * (-2.0 * (x - X0)) / SIGMA.powi(2);
        let v0 = -C * du_dx;

        // t=-Δt时的位移（泰勒展开近似：u(x,-Δt) ≈ u(x,0) - Δt·v(x,0)）
        u_prev[i] = u0 - dt * v0;
    }

    // 保存初始帧
    frames.push(u_curr.clone());

    // 时间推进（有限差分法求解波动方程）
    for _ in 1..T_STEPS {
        // 内部点更新（两端固定为0，无需计算）
        for i in 1..N-1 {
            u_next[i] = 2.0*(1.0 - R.powi(2))*u_curr[i] 
                - u_prev[i] 
                + R.powi(2)*(u_curr[i+1] + u_curr[i-1]);
        }

        // 保存当前帧并更新数组
        frames.push(u_next.clone());
        std::mem::swap(&mut u_prev, &mut u_curr);
        std::mem::swap(&mut u_curr, &mut u_next);
    }

    // 生成HTML可视化文件
    let x_data: Vec<f64> = (0..N).map(|i| i as f64 * dx).collect();
    let html = generate_html(&x_data, &frames, L, A);
    
    match write("wave_packet_simple.html", html) {
        Ok(_) => println!("模拟完成！文件已保存为 wave_packet_simple.html"),
        Err(e) => eprintln!("保存失败：{}", e),
    }
}

/// 生成简洁的HTML动态可视化页面
fn generate_html(x: &[f64], frames: &[Vec<f64>], l: f64, a: f64) -> String {
    let x_json = to_string(x).unwrap();
    let frames_json = to_string(frames).unwrap();
    let y_max = a * 1.3; // y轴余量，避免波形溢出

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>单向高斯波包传播（无分裂）</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ font-family: sans-serif; max-width: 1000px; margin: 20px auto; }}
        h1 {{ text-align: center; color: #2c3e50; }}
        .ctrl {{ text-align: center; margin: 15px 0; }}
        button {{ padding: 6px 12px; margin: 0 5px; cursor: pointer; }}
        canvas {{ width: 100% !important; height: 400px !important; }}
    </style>
</head>
<body>
    <h1>两端固定弦上的单向高斯波包（无分裂）</h1>
    <div class="ctrl">
        <button id="play">播放</button>
        <button id="pause">暂停</button>
        <button id="reset">重置</button>
        <span>速度：</span>
        <input type="range" id="speed" min="0.5" max="2" step="0.1" value="1">
    </div>
    <canvas id="waveChart"></canvas>

    <script>
        const x = {x_json};
        const frames = {frames_json};
        let currFrame = 0;
        let animId = null;
        let speed = 1.0;

        // 初始化图表
        const ctx = document.getElementById('waveChart').getContext('2d');
        const chart = new Chart(ctx, {{
            type: 'line',
            data: {{
                labels: x,
                datasets: [{{
                    label: '弦的位移',
                    data: frames[0],
                    borderColor: '#3498db',
                    borderWidth: 2,
                    pointRadius: 0,
                    tension: 0.2
                }}]
            }},
            options: {{
                scales: {{
                    x: {{ title: {{ display: true, text: '位置 (m)' }}, min: 0, max: {l} }},
                    y: {{ title: {{ display: true, text: '位移 (m)' }}, min: -{y_max}, max: {y_max} }}
                }},
                animation: {{ duration: 0 }}
            }}
        }});

        // 更新波形
        function update() {{
            chart.data.datasets[0].data = frames[currFrame];
            chart.update();
            currFrame = (currFrame + 1) % frames.length;
        }}

        // 动画控制
        function animate() {{
            update();
            animId = setTimeout(animate, 40 / speed);
        }}

        // 按钮事件
        document.getElementById('play').addEventListener('click', () => animId || animate());
        document.getElementById('pause').addEventListener('click', () => {{ clearTimeout(animId); animId = null; }});
        document.getElementById('reset').addEventListener('click', () => {{
            clearTimeout(animId); animId = null; currFrame = 0; update();
        }});
        document.getElementById('speed').addEventListener('input', (e) => speed = e.target.value);

        // 初始渲染
        update();
    </script>
</body>
</html>
        "#,
        x_json = x_json,
        frames_json = frames_json,
        l = l,
        y_max = y_max
    )
}
