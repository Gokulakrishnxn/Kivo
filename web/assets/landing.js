/* Canned examples only: this page never requests microphone access. */
(function () {
  const examples = {
    message: { mode: 'Message', input: '“um can you send me the notes after the meeting”', output: 'Can you send me the notes after the meeting' },
    standard: { mode: 'Standard', input: '“um the draft is ready and uh I will send it over this afternoon”', output: 'The draft is ready, and I will send it over this afternoon.' },
    raw: { mode: 'Raw', input: '“um let me think about that for a second”', output: 'um let me think about that for a second' }
  };
  const buttons = document.querySelectorAll('[data-example]');
  buttons.forEach(function (button) {
    button.addEventListener('click', function () {
      const example = examples[button.dataset.example];
      if (!example) return;
      document.getElementById('demo-input').textContent = example.input;
      document.getElementById('demo-output').textContent = example.output;
      document.getElementById('demo-mode').textContent = example.mode;
      buttons.forEach(function (item) { item.setAttribute('aria-pressed', String(item === button)); });
    });
  });
})();
