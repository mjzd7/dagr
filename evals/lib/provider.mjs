// Zero-dependency model providers. `mock` returns scripted patches so CI
// asserts harness mechanics deterministically; real providers need env keys.
export function makeProvider(name) {
  if (name === "mock") {
    return {
      name: "mock",
      async complete({ system, user }) {
        const raw = user.split("\n---TASK---\n")[1] ?? "";
        const jsonText = raw.includes("\nRespond with")
          ? raw.slice(0, raw.lastIndexOf("\nRespond with"))
          : raw;
        const task = JSON.parse(jsonText || "{}");
        return task.mock_response ?? "";
      },
    };
  }
  const key =
    name === "anthropic" ? process.env.ANTHROPIC_API_KEY :
    name === "openai" ? process.env.OPENAI_API_KEY : null;
  if (!key) throw new Error(`provider '${name}' requires an API key in env`);
  if (name === "anthropic") {
    return {
      name,
      async complete({ model = "claude-sonnet-4-20250514", system, user }) {
        const res = await fetch("https://api.anthropic.com/v1/messages", {
          method: "POST",
          headers: {
            "x-api-key": key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
          },
          body: JSON.stringify({
            model, max_tokens: 1024, system,
            messages: [{ role: "user", content: user }],
          }),
        });
        const data = await res.json();
        return {
          text: data.content?.[0]?.text ?? "",
          tokens_in: data.usage?.input_tokens ?? 0,
          tokens_out: data.usage?.output_tokens ?? 0,
        };
      },
    };
  }
  // openai
  return {
    name,
    async complete({ model = "gpt-4o-mini", system, user }) {
      const res = await fetch("https://api.openai.com/v1/chat/completions", {
        method: "POST",
        headers: { authorization: `Bearer ${key}`, "content-type": "application/json" },
        body: JSON.stringify({
          model, temperature: 0,
          messages: [{ role: "system", content: system }, { role: "user", content: user }],
        }),
      });
      const data = await res.json();
      return {
        text: data.choices?.[0]?.message?.content ?? "",
        tokens_in: data.usage?.prompt_tokens ?? 0,
        tokens_out: data.usage?.completion_tokens ?? 0,
      };
    },
  };
}
