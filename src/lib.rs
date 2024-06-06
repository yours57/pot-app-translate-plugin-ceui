use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use serde_json::json;

#[no_mangle]
pub fn translate(
    text: &str, // 待翻译文本
    from: &str, // 源语言
    to: &str,   // 目标语言
    // (pot会根据info.json 中的 language 字段传入插件需要的语言代码，无需再次转换)
    detect: &str, // 检测到的语言 (若使用 detect, 需要手动转换)
    needs: HashMap<String, String>, // 插件需要的其他参数,由info.json定义
) -> Result<Value, Box<dyn Error>> {

    let timeout = match needs.get("timeout") {
        Some(timeout) => timeout.parse::<u64>().unwrap_or(30),
        None => 30, // 默认超时时间为30秒
    };

    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(Duration::from_secs(timeout))
        .build()?;

    let apikey = match needs.get("apikey") {
        Some(apikey) => apikey.to_string(),
        None => return Err("apikey not found".into()),
    };

    let endpoint = match needs.get("endpoint") {
        Some(endpoint) => endpoint.to_string(),
        None => "https://api.openai.com/v1/chat/completions".to_string(),
    };

    let model = match needs.get("model") {
        Some(model) => model.to_string(),
        None => "gpt-4o".to_string(),
    };

    let prompt = match needs.get("prompt") {
        Some(prompt) => format!("{}\nOutput Language:{}", prompt, to),
        None => format!("Output Language:{}", to),
    };

    let stream = match needs.get("stream") {
        Some(stream) => stream.to_lowercase() == "true",
        None => false,
    };
    
    let temperature = match needs.get("temperature") {
        Some(temperature) => temperature.parse::<f64>().unwrap_or(0.5),
        None => 0.5,
    };
    
    let presence_penalty = match needs.get("presence_penalty") {
        Some(presence_penalty) => presence_penalty.parse::<f64>().unwrap_or(0.0),
        None => 0.0,
    };
    
    let frequency_penalty = match needs.get("frequency_penalty") {
        Some(frequency_penalty) => frequency_penalty.parse::<f64>().unwrap_or(0.0),
        None => 0.0,
    };
    
    let top_p = match needs.get("top_p") {
        Some(top_p) => top_p.parse::<f64>().unwrap_or(1.0),
        None => 1.0,
    };

    let request_body = json!({
        "messages": [
            {
                "role": "system",
                "content": prompt
            },
            {
                "role": "user",
                "content": text
            }
        ],
        "stream": stream,
        "model": model,
        "temperature": temperature,
        "presence_penalty": presence_penalty,
        "frequency_penalty": frequency_penalty,
        "top_p": top_p
    });

    let response = client
        .post(&endpoint)
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json")
        .header("authorization", format!("Bearer {}", apikey))
        .json(&request_body)
        .send()?;
    
    let response_text = response.text()?;
    let response_json: Value = serde_json::from_str(&response_text).map_err(|e| {
        eprintln!("Error decoding response body: {}", e);
        eprintln!("Response body: {}", response_text);
        "Response Parse Error"
    })?;
    
    // 检查是否包含错误字段
    if let Some(error) = response_json.get("error") {
        return Err(format!("API Error: {}", error).into());
    }
    
    // 提取正常响应字段
    match response_json["choices"][0]["message"]["content"].as_str() {
        Some(result) => Ok(Value::String(result.to_string())),
        None => Err("Response Parse Error".into()),
    }

    
}
