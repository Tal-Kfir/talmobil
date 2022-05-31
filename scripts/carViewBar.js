
const myProgressBar = document.querySelectorAll(".progress");

function updateProgressBar(progressBar) {
  for (let key of progressBar) {
      value = key.children[1].textContent.slice(0,-1);
      key.children[0].style.width = `${value}%`;
  }
}

updateProgressBar(myProgressBar);