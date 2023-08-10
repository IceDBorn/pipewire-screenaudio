export default function Button () {
  function handleClick () {
    console.log('test')
  }

  return (
    <div>
      <button onClick={handleClick}>test</button>
    </div>
  )
}
