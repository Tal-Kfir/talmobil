const form = document.querySelector("form");
form.addEventListener("submit", (e) => {
	e.preventDefault();
	const formData = new FormData(form);
	console.log(form);
	axios
      .post("/edit-car/" + window.location.toString().split('/').pop(), formData, {
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



function deleteCar() {
	axios.post('/deleteCar/' + window.location.toString().split('/').pop(), {
			withCredentials: true,
		  	})
		  .then((res) => {
        alert("Success");
		window.location = "/home";
      })
      .catch((err) => {
        alert("Failed to Submit");
      });
  };