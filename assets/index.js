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