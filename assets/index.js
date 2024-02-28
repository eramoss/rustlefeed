document.addEventListener('DOMContentLoaded', function () {
  const contentContainer = document.getElementById('feed-item');
  const likeBtn = document.getElementById('like');
  const dislikeBtn = document.getElementById('dislike');

  async function fetchNextContent(is_liked) {
    const response = await fetch('/next', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ liked: is_liked })
    });
    if (response.ok) {
      window.scrollTo({ top: 0, behavior: 'smooth' });
      const content = await response.text();
      setTimeout(() => {
        contentContainer.innerHTML = content;
      }, 200);
    } else {
      console.error('Failed to fetch next content');
    }
  }

  likeBtn.addEventListener('click', function () {
    fetchNextContent(true);
  });
  dislikeBtn.addEventListener('click', function () {
    fetchNextContent(false);
  });
  fetchNextContent(null);
});
function sendFeed(url) {
  fetch('/add-feed', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ url: url })
  })
  .then(response => {
    if (response.ok) {
      return response.json();
    }
    displayErrorMessageInsideInput(document.getElementById('add-feed-input'), 'Feed not found');
  })
  .catch(error => {
    // Handle error
    console.error('Error:', error);
    displayErrorMessage('There was an error processing your request.');
  });
}

function displayErrorMessageInsideInput(input, message) {
  const errorMessage = input.nextElementSibling;
  if (errorMessage && errorMessage.classList.contains('error-message')) {
    errorMessage.textContent = message;
  } else {
    const errorDiv = document.createElement('div');
    errorDiv.classList.add('error-message');
    errorDiv.textContent = message;
    input.parentNode.insertBefore(errorDiv, input.nextSibling);
    setTimeout(() => {
      errorDiv.remove();
    }, 2000); // Remove error message after 2 seconds
  }
}

// Event listener for Enter key press
document.getElementById('add-feed-input').addEventListener('keypress', function(event) {
  if (event.key === 'Enter') {
    const url = event.target.value.trim();
    const input = event.target;
    if (url !== '') {
      sendFeed(url);
      event.target.value = '';
      const errorMessage = input.nextElementSibling;
      if (errorMessage && errorMessage.classList.contains('error-message')) {
        errorMessage.parentNode.removeChild(errorMessage);
      }
    } else {
      console.error('URL cannot be empty');
      displayErrorMessageInsideInput(input, 'URL cannot be empty');
    }
  }
});