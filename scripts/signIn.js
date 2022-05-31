

function onSignIn(googleUser) {
        // Useful data for your client-side scripts:
        var profile = googleUser.getBasicProfile();
        console.log("ID: " + profile.getId()); // Don't send this directly to your server!
        console.log('Full Name: ' + profile.getName());
        console.log('Given Name: ' + profile.getGivenName());
        console.log('Family Name: ' + profile.getFamilyName());
        console.log("Image URL: " + profile.getImageUrl());
        console.log("Email: " + profile.getEmail());
		
		

        // The ID token you need to pass to your backend:
        var id_token = googleUser.getAuthResponse().id_token;
        console.log("ID Token: " + id_token);
		
		axios.post('/login', {
				Full_Name: profile.getName(),
				Given_Name: profile.getGivenName(),
				Email: profile.getEmail(),
				Token: id_token
				
			  })
			  .then(function (response) {
				console.log(response);
				console.log(response.data.toString())
				if (response.data.toString().startsWith("Success")) {
					window.location = "./meow.html";
				}
			  })
			  .catch(function (error) {
				console.log(error);
			  });
}