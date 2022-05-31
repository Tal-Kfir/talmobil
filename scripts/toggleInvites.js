
function toggleInvites() {
	
	axios.post('/toggleInvites', {
			withCredentials: true,
		  	})
		  .then(function (response) {
			console.log(response);
			console.log(response.data.toString())
			if (response.data.toString().startsWith("Success")) {
				console.log("Updated Invites");
			}
		  })
		  .catch(function (error) {
			console.log(error);
		  });
}