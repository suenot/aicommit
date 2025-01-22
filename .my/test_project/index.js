import OpenAI from 'openai';

const openai = new OpenAI({
  // You can get a token in the bot https://t.me/DeepGPTBot calling the `/api` command
  apiKey: "0918aaf06c5ea56103e4f67232a19c82", 
  baseURL: "https://api.deep-foundation.tech/v1/"
});

async function main() {
  const chatCompletion = await openai.chat.completions.create({
    messages: [{ role: 'user', content: 'How are you? Ответь мне на русском языке' }],
    model: 'gpt-4o-mini',
  });
  
  console.log(chatCompletion.choices[0].message.content);
}

main();