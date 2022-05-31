const actualBtn = document.getElementById('actual-btn');
const fileChosen = document.getElementById('file-chosen');

console.log(actualBtn);
console.log(fileChosen);
actualBtn.addEventListener('change', function() {
  fileChosen.textContent = this.files[0].name
})

const form = document.querySelector("form");
form.addEventListener("submit", (e) => {
	e.preventDefault();
	const formData = new FormData(form);
	console.log(form);
	axios
      .post("/add-car", formData, {
		withCredentials: true,
        headers: {
          "Content-Type": "multipart/form-data",
        },
      })
      .then((res) => {
        alert("Success");
		window.location = "/home";
      })
      .catch((err) => {
        alert("Failed to Submit");
      });
  });