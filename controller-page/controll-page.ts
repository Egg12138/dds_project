const controlForm = 
  document.getElementById('controlForm') as HTMLFormElement;
const resetForm = 
  document.getElementById('resetForm') as HTMLFormElement;
// monitorXX will be used as the XXinput
// const monitorFreq = document.getElementById('monitorForm')!.querySelector('input[name="monitor_freq"]') as HTMLInputElement;
// const monitorVoltage = document.getElementById('monitorForm')!.querySelector('input[name="monitor_volt"]') as HTMLInputElement;
const mfreq = document.getElementsByName('monitor_freq')[0] as HTMLInputElement;
const mamp = document.getElementsByName('monitor_amp')[0] as HTMLInputElement;

function delay(ms: number) {
  return new Promise( resolve => setTimeout(resolve, ms))
}



controlForm.addEventListener('submit', 
  async (event) => {
  event.preventDefault(); // Prevent default form submission
  console.log("-----------------")

  const formData = new FormData(controlForm);
  const freq = formData.get('freq');
  const amp = formData.get('amp');

  console.log("simulating...do some thing");
  console.log(freq)
  console.log(amp)
  // Send data to Rust backend 
  // TODO: example...
  const response = await fetch('/api/set_values', {
    method: 'POST',
    body: JSON.stringify({ freq, amp }),
    headers: { 'Content-Type': 'application/json' }
  });

    // await delay(10000);
  // const response = {
  //   method: 'POST',
  //   body: JSON.stringify({ freq, amp}),
  //   headers: {
  //     'Content-Type': 'application/json'
  //   }
  // }

  if (response.ok) {
  // const data = JSON.parse(response.body)
    const data = await response.json();
  // Update monitor form with received data (assuming response contains freq and volt)
    mfreq.value = data.freq;
    mamp.value = data.amp;
      } else {
    // Handle errors from the backend
    console.error('Error sending data:', response.statusText);
  }
});

resetForm.addEventListener('reset', 
(event) => {
  event.preventDefault();
  // Reset form values
  let userinput_freqs = document.getElementsByName('freq');
  let userinput_amps = document.getElementsByName('amp');
  for (let i = 0; i < userinput_freqs.length; i++) {
    (userinput_freqs[i] as HTMLInputElement).value = "1";
  }
  for (let i = 0; i < userinput_amps.length; i++) {
    (userinput_amps[i] as HTMLInputElement).value = "1";
  }
});
